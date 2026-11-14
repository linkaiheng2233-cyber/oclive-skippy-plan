#include "gs_wire.h"

#ifdef GS_KERNEL_ENABLED
#include "kernel_bridge.h"
#endif

namespace gunspirit::wire {
namespace {

constexpr uint8_t kMagic0 = 0x47;  // 'G'
constexpr uint8_t kMagic1 = 0x53;  // 'S'
constexpr uint8_t kVersion = 1;
constexpr size_t kHeaderLen = 7;
constexpr size_t kCrcLen = 2;
constexpr size_t kMaxPayload = 4096;
constexpr size_t kMaxFrame = kHeaderLen + kMaxPayload + kCrcLen;
constexpr uint8_t kStateFormatVersion = 1;
constexpr uint16_t kContractMajor = 0;
constexpr uint16_t kContractMinor = 4;

// 消息类型：数值即协议，与 Rust 侧一致。
enum MessageType : uint8_t {
    kHello = 0x01,
    kState = 0x02,
    kEvent = 0x03,
    kExpression = 0x04,
    kAck = 0x05,
    kNack = 0x06,
    kPing = 0x07,
    kPong = 0x08,
    kAim = 0x09,
};

enum NackCode : uint8_t {
    kNackUnsupportedType = 1,
    kNackBadPayload = 2,
    kNackUnsupportedVersion = 3,
    kNackNotReady = 4,
    kNackBusy = 5,
};

bool is_known_type(uint8_t type) {
    return type >= kHello && type <= kAim;
}

bool s_active = false;
uint32_t s_frames_ok = 0;
uint32_t s_frames_bad = 0;
bool s_peer_seen = false;
uint64_t s_revision = 0;
uint32_t s_last_state_ms = 0;

// 启动标识：用 MAC 后 3 字节 + 启动计数，满足契约 1..=128 字节上限。
char s_boot_id[32] = "board-boot-0";

// 接收缓冲（容纳一整个最大帧）。
uint8_t s_buffer[kMaxFrame];
size_t s_length = 0;

size_t serial_writer(const uint8_t *data, size_t length) {
    return Serial.write(data, length);
}

FrameWriter s_writer = nullptr;

size_t write_bytes(const uint8_t *data, size_t length) {
    if (s_writer != nullptr) {
        return s_writer(data, length);
    }
    return serial_writer(data, length);
}

uint16_t crc16_ccitt_false(const uint8_t *data, size_t length) {
    uint16_t crc = 0xFFFF;
    for (size_t index = 0; index < length; ++index) {
        crc ^= static_cast<uint16_t>(data[index]) << 8;
        for (int bit = 0; bit < 8; ++bit) {
            crc = (crc & 0x8000U) != 0U ? static_cast<uint16_t>((crc << 1) ^ 0x1021U)
                                        : static_cast<uint16_t>(crc << 1);
        }
    }
    return crc;
}

void write_u16(uint8_t *buffer, uint16_t value) {
    buffer[0] = static_cast<uint8_t>(value & 0xFF);
    buffer[1] = static_cast<uint8_t>((value >> 8) & 0xFF);
}

void write_u64(uint8_t *buffer, uint64_t value) {
    for (int index = 0; index < 8; ++index) {
        buffer[index] = static_cast<uint8_t>((value >> (8 * index)) & 0xFF);
    }
}

uint16_t read_u16(const uint8_t *buffer) {
    return static_cast<uint16_t>(buffer[0] | (static_cast<uint16_t>(buffer[1]) << 8));
}

uint32_t read_u32(const uint8_t *buffer) {
    return static_cast<uint32_t>(buffer[0]) | (static_cast<uint32_t>(buffer[1]) << 8) |
           (static_cast<uint32_t>(buffer[2]) << 16) | (static_cast<uint32_t>(buffer[3]) << 24);
}

/// 组帧并直接写出。返回 false 表示长度溢出。
bool send_frame(uint8_t type, const uint8_t *payload, size_t length) {
    if (length > kMaxPayload) {
        return false;
    }
    uint8_t frame[kHeaderLen + kMaxPayload + kCrcLen];
    frame[0] = kMagic0;
    frame[1] = kMagic1;
    frame[2] = kVersion;
    frame[3] = type;
    frame[4] = 0;
    write_u16(&frame[5], static_cast<uint16_t>(length));
    if (length > 0) {
        memcpy(&frame[kHeaderLen], payload, length);
    }
    const uint16_t crc = crc16_ccitt_false(frame, kHeaderLen + length);
    write_u16(&frame[kHeaderLen + length], crc);
    const size_t total = kHeaderLen + length + kCrcLen;
    return write_bytes(frame, total) == total;
}

void send_hello() {
    uint8_t payload[160];
    size_t offset = 0;
    write_u16(&payload[offset], kContractMajor);
    offset += 2;
    write_u16(&payload[offset], kContractMinor);
    offset += 2;
    const size_t boot_len = strlen(s_boot_id);
    payload[offset++] = static_cast<uint8_t>(boot_len);
    memcpy(&payload[offset], s_boot_id, boot_len);
    offset += boot_len;
    write_u16(&payload[offset], 0x001F);  // 能力位图占位：低 5 位。
    offset += 2;
    write_u16(&payload[offset], 0);  // 高位补零，保持 u32 布局。
    offset += 2;
    send_frame(kHello, payload, offset);
}

void send_pong(uint32_t nonce) {
    uint8_t payload[4];
    payload[0] = static_cast<uint8_t>(nonce & 0xFF);
    payload[1] = static_cast<uint8_t>((nonce >> 8) & 0xFF);
    payload[2] = static_cast<uint8_t>((nonce >> 16) & 0xFF);
    payload[3] = static_cast<uint8_t>((nonce >> 24) & 0xFF);
    send_frame(kPong, payload, sizeof(payload));
}

void send_ack(uint32_t sequence) {
    uint8_t payload[4];
    payload[0] = static_cast<uint8_t>(sequence & 0xFF);
    payload[1] = static_cast<uint8_t>((sequence >> 8) & 0xFF);
    payload[2] = static_cast<uint8_t>((sequence >> 16) & 0xFF);
    payload[3] = static_cast<uint8_t>((sequence >> 24) & 0xFF);
    send_frame(kAck, payload, sizeof(payload));
}

void send_nack(uint32_t sequence, uint8_t code) {
    uint8_t payload[5];
    payload[0] = static_cast<uint8_t>(sequence & 0xFF);
    payload[1] = static_cast<uint8_t>((sequence >> 8) & 0xFF);
    payload[2] = static_cast<uint8_t>((sequence >> 16) & 0xFF);
    payload[3] = static_cast<uint8_t>((sequence >> 24) & 0xFF);
    payload[4] = code;
    send_frame(kNack, payload, sizeof(payload));
}

/// 发布 STATE。M3 阶段**全部 fact 为 Unknown**：内核尚未上板，
/// 伪造"看起来正常"的事实会污染主机判定，宁可显式 Unknown。
void publish_state(uint32_t now_ms) {
    // 内核就绪时，STATE 来自真实融合快照（同一份 Rust 编码器输出）。
    // 缓冲区放静态区而不是栈：loopTask 栈余量只有数 KB。
    static uint8_t s_state_payload[512];
#ifdef GS_KERNEL_ENABLED
    if (gs_kernel_ready() == 1) {
        gs_kernel_tick(static_cast<uint64_t>(now_ms));
        const int32_t written = gs_kernel_encode_state(s_state_payload, sizeof(s_state_payload));
        if (written > 0) {
            send_frame(kState, s_state_payload, static_cast<size_t>(written));
            return;
        }
        Serial.printf("WIRE ERR KERNEL_ENCODE %ld\n", static_cast<long>(written));
    }
#endif
    uint8_t payload[320];
    size_t offset = 0;
    payload[offset++] = kStateFormatVersion;
    write_u16(&payload[offset], kContractMajor);
    offset += 2;
    write_u16(&payload[offset], kContractMinor);
    offset += 2;
    const size_t boot_len = strlen(s_boot_id);
    payload[offset++] = static_cast<uint8_t>(boot_len);
    memcpy(&payload[offset], s_boot_id, boot_len);
    offset += boot_len;
    write_u64(&payload[offset], ++s_revision);
    offset += 8;
    write_u64(&payload[offset], static_cast<uint64_t>(now_ms));
    offset += 8;
    // 六个 fact：tag=0(Unknown)、reason=0(NeverObserved)、since_revision。
    for (int fact = 0; fact < 6; ++fact) {
        payload[offset++] = 0;
        payload[offset++] = 0;
        write_u64(&payload[offset], s_revision);
        offset += 8;
    }
    payload[offset++] = 0;  // node_count = 0
    send_frame(kState, payload, offset);
}

void dispatch(uint8_t type, const uint8_t *payload, size_t length) {
    switch (type) {
        case kHello: {
            if (length >= 5) {
                const uint8_t boot_len = payload[4];
                if (boot_len > 0 && length >= static_cast<size_t>(5 + boot_len + 4)) {
                    const uint16_t peer_major = read_u16(&payload[0]);
                    const uint16_t peer_minor = read_u16(&payload[2]);
                    if (peer_major != kContractMajor || peer_minor != kContractMinor) {
                        send_nack(0, kNackUnsupportedVersion);
                        return;
                    }
                    s_peer_seen = true;
                    send_hello();
                    publish_state(millis());
                    return;
                }
            }
            send_nack(0, kNackBadPayload);
            return;
        }
        case kPing: {
            if (length >= 4) {
                send_pong(read_u32(payload));
            } else {
                send_nack(0, kNackBadPayload);
            }
            return;
        }
        case kExpression: {
            if (length < 2) {
                send_nack(0, kNackBadPayload);
                return;
            }
            const uint8_t name_len = payload[0];
            if (name_len == 0 || length < static_cast<size_t>(1 + name_len)) {
                send_nack(0, kNackBadPayload);
                return;
            }
            char name[33] = {0};
            const size_t copy_len = name_len < 32 ? name_len : 32;
            memcpy(name, &payload[1], copy_len);
            if (!apply_mood(name)) {
                send_nack(0, kNackBadPayload);
                return;
            }
            send_ack(0);
            publish_state(millis());
            return;
        }
        case kState:
        case kEvent:
        case kAck:
        case kPong:
            // 这些方向本应是板 → 主机；收到不做处理。
            return;
        default:
            send_nack(0, kNackUnsupportedType);
            return;
    }
}

/// 尝试从缓冲区头部取出一帧。返回消耗的字节数（0 表示需要更多数据）。
size_t take_frame() {
    if (s_length < kHeaderLen) {
        return 0;
    }
    if (s_buffer[0] != kMagic0 || s_buffer[1] != kMagic1) {
        return 1;  // 滑动一字节重同步
    }
    if (s_buffer[2] != kVersion) {
        ++s_frames_bad;
        return 1;
    }
    if (!is_known_type(s_buffer[3])) {
        ++s_frames_bad;
        return 1;
    }
    const size_t payload_len = read_u16(&s_buffer[5]);
    if (payload_len > kMaxPayload) {
        return 1;
    }
    const size_t total = kHeaderLen + payload_len + kCrcLen;
    if (s_length < total) {
        return 0;
    }
    const uint16_t expected = read_u16(&s_buffer[kHeaderLen + payload_len]);
    const uint16_t actual = crc16_ccitt_false(s_buffer, kHeaderLen + payload_len);
    if (expected != actual) {
        ++s_frames_bad;
        Serial.printf("WIRE ERR CRC expected=0x%04X actual=0x%04X\n", expected, actual);
        return total;
    }
    ++s_frames_ok;
    dispatch(s_buffer[3], &s_buffer[kHeaderLen], payload_len);
    return total;
}

}  // namespace

void begin() {
    s_active = true;
    s_peer_seen = false;
    s_length = 0;
    s_revision = 0;
    snprintf(s_boot_id, sizeof(s_boot_id), "board-%02X%02X%02X-1",
             static_cast<unsigned>(ESP.getEfuseMac() >> 32 & 0xFF),
             static_cast<unsigned>(ESP.getEfuseMac() >> 24 & 0xFF),
             static_cast<unsigned>(ESP.getEfuseMac() >> 16 & 0xFF));
    Serial.printf("WIRE on boot_id=%s\n", s_boot_id);
}

void end() {
    s_active = false;
    s_length = 0;
    Serial.println("WIRE off");
}

bool active() {
    return s_active;
}

bool peer_seen() {
    return s_peer_seen;
}

bool publish_event(const uint8_t *payload, size_t length) {
    return send_frame(kEvent, payload, length);
}

uint32_t frames_received() {
    return s_frames_ok;
}

uint32_t frame_errors() {
    return s_frames_bad;
}

void set_frame_writer(FrameWriter writer) {
    s_writer = writer;
}

void feed(uint8_t byte) {
    if (s_length >= sizeof(s_buffer)) {
        // 溢出保护：丢掉最旧的一个字节再接收。
        memmove(s_buffer, &s_buffer[1], sizeof(s_buffer) - 1);
        --s_length;
    }
    s_buffer[s_length++] = byte;
    for (;;) {
        const size_t consumed = take_frame();
        if (consumed == 0) {
            return;
        }
        if (consumed >= s_length) {
            s_length = 0;
            return;
        }
        memmove(s_buffer, &s_buffer[consumed], s_length - consumed);
        s_length -= consumed;
    }
}

void pump(uint32_t now_ms) {
    if (!s_active || !s_peer_seen) {
        return;
    }
    if (now_ms - s_last_state_ms < 1000) {
        return;
    }
    s_last_state_ms = now_ms;
    publish_state(now_ms);
}

}  // namespace gunspirit::wire







