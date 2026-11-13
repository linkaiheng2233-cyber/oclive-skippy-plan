/* V1 链接层验证：C 侧直接调用 Rust 感知内核的 C ABI。
 *
 * 只要能编译并链接出目标文件，就证明：
 * 1. Rust 静态库在真 Xtensa 目标上可链接；
 * 2. panic_handler / global_allocator / C ABI 三者是一次完整链路；
 * 3. 固件侧不需要知道任何 Rust 细节。
 */
#include <stdint.h>

extern uint32_t gs_kernel_abi_version(void);
extern int32_t gs_kernel_self_test(void);
extern uint64_t gs_kernel_state_revision(void);

int main(void)
{
    if (gs_kernel_abi_version() != 1u) {
        return 1;
    }
    int32_t revision = gs_kernel_self_test();
    if (revision < 0) {
        return 2;
    }
    (void)gs_kernel_state_revision();
    return revision;
}
