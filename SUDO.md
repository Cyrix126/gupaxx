## Sudo
Unlike Windows, Unix (macOS/Linux) has a userland program that handles all the dirty details of privilege escalation: `sudo`.

`sudo` is used in Gupax to execute XMRig, to enable MSR mods and hugepages. After every use of `sudo`, the memory holding the `String` buffer containing the password is wiped with 0's using [`zeroize`](https://docs.rs/zeroize/) to make sure the compiler doesn't optimize away the wipe. Although memory *should* be safe, this prevents passive accidents (core-dumps revealing plain-text password) and active attacks (attackers accessing live process memory) from happening.

In general, secrets should be ephemeral or encrypted if not in use. I considered using [`secrets`](https://docs.rs/secrets/) to keep the password encrypted so that the user would only have to enter their password once, but simply wiping the memory was easier to implement and caused less worries of handling things incorrectly.

If the user is allowed to use sudo without a password, there will be no prompt.
