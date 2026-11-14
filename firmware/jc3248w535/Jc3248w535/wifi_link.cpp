#include "wifi_link.h"

#include <Preferences.h>
#include <WiFi.h>
#include <WiFiUdp.h>

#include "gs_wire.h"

namespace gunspirit::wifi_link {
namespace {

constexpr uint16_t kTcpPort = 47900;
constexpr uint16_t kDiscoveryPort = 47901;
constexpr uint32_t kReconnectIntervalMs = 5000;
constexpr uint32_t kAnnounceIntervalMs = 3000;
constexpr const char *kDiscoverRequest = "GS-DISCOVER";
constexpr const char *kNvsNamespace = "gunspirit";

Preferences s_preferences;
WiFiServer s_server(kTcpPort);
WiFiUDP s_udp;
WiFiClient s_client;

String s_ssid;
String s_password;
bool s_has_credentials = false;
bool s_started = false;
uint32_t s_last_attempt_ms = 0;
uint32_t s_last_announce_ms = 0;

bool load_credentials() {
    if (!s_preferences.begin(kNvsNamespace, true)) {
        return false;
    }
    s_ssid = s_preferences.getString("ssid", "");
    s_password = s_preferences.getString("pass", "");
    s_preferences.end();
    s_has_credentials = !s_ssid.isEmpty();
    return s_has_credentials;
}

bool store_credentials(const String &ssid, const String &password) {
    if (!s_preferences.begin(kNvsNamespace, false)) {
        return false;
    }
    s_preferences.putString("ssid", ssid);
    s_preferences.putString("pass", password);
    s_preferences.end();
    s_ssid = ssid;
    s_password = password;
    s_has_credentials = true;
    return true;
}

void forget_credentials() {
    if (s_preferences.begin(kNvsNamespace, false)) {
        s_preferences.remove("ssid");
        s_preferences.remove("pass");
        s_preferences.end();
    }
    s_ssid = "";
    s_password = "";
    s_has_credentials = false;
}

/// 把帧写到 socket；返回值用于 gs_wire 判断是否写成功。
size_t socket_writer(const uint8_t *data, size_t length) {
    if (!s_client.connected()) {
        return 0;
    }
    return s_client.write(data, length);
}

void start_server() {
    if (s_started) {
        return;
    }
    s_server.begin();
    s_server.setNoDelay(true);
    s_udp.begin(kDiscoveryPort);
    s_started = true;
    // 连接后把帧出口切到 socket；断开时再切回串口。
    wire::set_frame_writer(nullptr);
    Serial.printf("WIFI connected ip=%s tcp=%u udp=%u\n",
                  WiFi.localIP().toString().c_str(), static_cast<unsigned>(kTcpPort),
                  static_cast<unsigned>(kDiscoveryPort));
}

void stop_server() {
    if (!s_started) {
        return;
    }
    s_client.stop();
    s_server.end();
    s_udp.stop();
    s_started = false;
    wire::set_frame_writer(nullptr);
}

void service_discovery() {
    const int packet_size = s_udp.parsePacket();
    if (packet_size <= 0) {
        return;
    }
    char request[32] = {0};
    const int read = s_udp.read(request, sizeof(request) - 1);
    if (read <= 0 || strncmp(request, kDiscoverRequest, strlen(kDiscoverRequest)) != 0) {
        return;
    }
    const String reply = String("GS-HERE ") + WiFi.localIP().toString() + ":" + String(kTcpPort);
    s_udp.beginPacket(s_udp.remoteIP(), s_udp.remotePort());
    s_udp.write(reinterpret_cast<const uint8_t *>(reply.c_str()), reply.length());
    s_udp.endPacket();
}

}  // namespace

void begin() {
    if (!load_credentials()) {
        Serial.println("WIFI no credentials (use: WIFI <ssid> <pass>)");
        return;
    }
    WiFi.mode(WIFI_STA);
    WiFi.setAutoReconnect(true);
    WiFi.begin(s_ssid.c_str(), s_password.c_str());
    s_last_attempt_ms = millis();
    Serial.printf("WIFI connecting ssid=%s\n", s_ssid.c_str());
}

void pump() {
    if (!s_has_credentials) {
        return;
    }
    const uint32_t now = millis();

    if (WiFi.status() != WL_CONNECTED) {
        if (s_started) {
            Serial.println("WIFI lost");
            stop_server();
        }
        // 只在"已断开/连接失败/找不到网络"时重试；连接中不打断握手，
        // 也不再调用 disconnect()——否则会打断正在进行的关联。
        const wl_status_t status = WiFi.status();
        const bool retryable = status == WL_DISCONNECTED || status == WL_CONNECT_FAILED ||
                               status == WL_NO_SSID_AVAIL || status == WL_IDLE_STATUS;
        if (retryable && now - s_last_attempt_ms >= kReconnectIntervalMs) {
            s_last_attempt_ms = now;
            Serial.printf("WIFI retry status=%d\n", static_cast<int>(status));
            WiFi.begin(s_ssid.c_str(), s_password.c_str());
        }
        return;
    }

    if (!s_started) {
        start_server();
    }

    service_discovery();

    if (!s_client.connected()) {
        WiFiClient candidate = s_server.available();
        if (candidate) {
            s_client = candidate;
            s_client.setNoDelay(true);
            s_udp.flush();
            wire::set_frame_writer(socket_writer);
            Serial.printf("WIFI client %s\n", s_client.remoteIP().toString().c_str());
        }
        return;
    }

    // 把 socket 收到的字节喂进协议解析器（与串口同一套）。
    for (int budget = 256; budget > 0 && s_client.available() > 0; --budget) {
        const int byte = s_client.read();
        if (byte < 0) {
            break;
        }
        wire::feed(static_cast<uint8_t>(byte));
    }

    // 周期性广播自身地址。**由板子主动广播**而不是等电脑来问：
    // 电脑可能同时有多个网卡，向 255.255.255.255 的请求只会走默认路由，
    // 未必落在热点子网上；而板子就站在热点里，它的广播一定到得了电脑。
    if (now - s_last_announce_ms >= kAnnounceIntervalMs) {
        s_last_announce_ms = now;
        const String announcement =
            String("GS-HERE ") + WiFi.localIP().toString() + ":" + String(kTcpPort);
        s_udp.beginPacket(IPAddress(255, 255, 255, 255), kDiscoveryPort);
        s_udp.write(reinterpret_cast<const uint8_t *>(announcement.c_str()), announcement.length());
        s_udp.endPacket();
        Serial.printf("WIFI announce %s\n", announcement.c_str());
    }
}

bool command(const char *line) {
    if (strncmp(line, "WIFI", 4) != 0) {
        return false;
    }
    const char *argument = line + 4;
    while (*argument == ' ') {
        ++argument;
    }
    if (*argument == '\0' || strcmp(argument, "status") == 0) {
        Serial.printf("WIFI status connected=%d ip=%s client=%d ssid=%s\n",
                      WiFi.status() == WL_CONNECTED ? 1 : 0, ip(),
                      s_client.connected() ? 1 : 0, s_has_credentials ? s_ssid.c_str() : "");
        return true;
    }
    if (strcmp(argument, "scan") == 0) {
        const int found = WiFi.scanNetworks(false, false);
        Serial.printf("WIFI scan found=%d\n", found);
        for (int index = 0; index < found; ++index) {
            Serial.printf("  AP %d: ssid=%s rssi=%d channel=%d enc=%d\n", index,
                          WiFi.SSID(index).c_str(), WiFi.RSSI(index), WiFi.channel(index),
                          static_cast<int>(WiFi.encryptionType(index)));
        }
        WiFi.scanDelete();
        return true;
    }
    if (strcmp(argument, "forget") == 0) {
        forget_credentials();
        WiFi.disconnect();
        stop_server();
        Serial.println("WIFI forgot");
        return true;
    }
    const char *separator = strchr(argument, ' ');
    if (separator == nullptr) {
        Serial.println("ERR WIFI need: WIFI <ssid> <pass> | WIFI status | WIFI forget");
        return true;
    }
    String ssid(argument);
    ssid = ssid.substring(0, static_cast<unsigned>(separator - argument));
    String password(separator + 1);
    if (ssid.isEmpty() || password.isEmpty()) {
        Serial.println("ERR WIFI empty ssid or pass");
        return true;
    }
    if (!store_credentials(ssid, password)) {
        Serial.println("ERR WIFI nvs write failed");
        return true;
    }
    WiFi.disconnect();
    WiFi.mode(WIFI_STA);
    WiFi.begin(s_ssid.c_str(), s_password.c_str());
    s_last_attempt_ms = millis();
    // 不回显密码，只报告 SSID。
    Serial.printf("WIFI saved ssid=%s\n", s_ssid.c_str());
    return true;
}

bool connected() {
    return WiFi.status() == WL_CONNECTED;
}

bool client_connected() {
    return s_client.connected();
}

const char *ip() {
    static String cached;
    cached = WiFi.localIP().toString();
    return cached.c_str();
}

}  // namespace gunspirit::wifi_link



