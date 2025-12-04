## Why does Gupax need to be Admin? (on Windows)
**TL;DR:** Because Windows.  

**Slightly more detailed TL;DR:** Rust does not have mature Win32 API wrapper libraries. Although Microsoft has an official ["Rust" library](https://github.com/microsoft/windows-rs), it is quite low-level and using it within Gupax would mean re-implementing a lot of Rust's STDLIB process module code.

If you are confused because you use Gupax on macOS/Linux, this is a Windows-only issue.

The following sections will go more into the technical issues I've encountered in trying to implement something that sounds pretty trivial: Starting a child process with elevated privilege, and getting a handle to it and its output. (it's a rant about windows).

---

### The issue
`XMRig` needs to be run with administrative privileges to enable MSR mods and hugepages. There are other ways of achieving this through pretty manual and technical efforts (which also gets more complicated due to OS differences) but in the best interest of Gupax's users, I always want to implement things so that it's **easy for the user.**

Users should not need to be familiar with MSRs to get max hashrate, this is something the program (me, Gupax!) should do for them.

---

### The requirements
Process's in Gupax need the following criteria met:
- I (as the parent process, Gupax) *must* have a direct handle to the process so that I can send SIGNALs
- I *must* have a handle to the process's STDOUT+STDERR so that I can actually relay output to the user
- I *really should* but don't absolutely need a handle to STDIN so that I can send input from the user

In the case of XMRig, **I absolutely must enable MSR's automatically for the user**, that's the whole point of XMRig, that's the point of an easy-to-use GUI.
Although I want XMRig with elevated rights, I don't want these side-effects:
- All of Gupax running as Admin
- P2Pool running as Admin

Here are the "solutions" I've attempted:

---

### CMD's RunAs
Window has a `runas` command, which allows for privilege escalation. Perfect! Spawn a shell and it's easy as running this:
```
runas /user:Administrator xmrig.exe [...]
```
...right?

The `Administrator` in this context is a legacy account, not meant to be touched, not really the `Admin` we are looking for, but more importantly: the password is not set, and the entire account is disabled by default. This means you cannot actually `runas` as *that* `Administrator`. Technically, all it would take is for the user to enabled the account and set a password. But that is already asking for too much, remember: that's my job, to make this **easy and automatic**. So this is a no-go, next.

---

### PowerShell's Start-Process
Window's `PowerShell` has a nice built-in called [`Start-Process`](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/start-process?view=powershell-7.3). This allows PowerShell to start... processes. In particular, I was intrigued by the all-in-one flag: `-Verb RunAs`, which runs the provided process with elevated permissions after a **UAC prompt.** That sounds perfect... except if you click that link you'll see 2 sets of syntax. IF you are escalating privilege, Microsoft puts a lot more retrictions on what you can do with this built-in, in particular:
- You CANNOT redirect STDOUT/STDERR/STDIN
- You CANNOT run the process in the current shell (a new PowerShell window will always open!)

I attempted some hacks like chaining non-admin PowerShell + admin PowerShell together, which made things overly complicated and meant I would be handling logic within these child PowerShell's which would be controlled via STDIN from Gupax code... Not very robust. I also tried just starting an admin PowerShell directly from Gupax, but that meant the user, upon clicking `[Start]` for XMRig, would see a UAC prompt to open PowerShell, which wasn't a good look. Eventually I gave up on PowerShell, next.

---

### Win32's ShellExecuteW
This was the first option I came across, but I intentionally ignored it due to many reasons. Microsoft has official Windows API bindings in [Rust](https://github.com/microsoft/windows-rs). That library has a couple problems:
1. All (the entire library) code requires `unsafe`
2. It's extremely low-level

The first one isn't actually as bad as it seems, this is Win32 so it's battle-tested. It's also extern C, so it makes sense it has to wrapped in `unsafe`.

The second one is the real issue. [ShellExecuteW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew) is a Win32 function that allows exactly what I need, starting a process with elevated privilege with the `runas` flag. It even shows the UAC to the user. But... that's it! No other functionality. The highly abstracted `Command` type in Rust's STDLIB actually uses [`CreateProcessW`](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw), and due to type imcompatabilities, using `ShellExecuteW` on my own would mean re-implementing ALL the functionality Rust STDLIB gives, aka: handling STDOUT, STDERR, STDIN, sending SIGNALS, waiting on process, etc etc. I would be programming for "Windows", not "Rust". Okay... next.

---

### Registry Edit
To start a process in Windows with elevated escalation you can right-click -> `Run as Administrator`, but you can also set a permanent flag deeper in the file's options. In reality this sets a Registry Key with the absolute path to that executable and a `RUNASADMIN` flag. This allows Windows to know which programs to run as an admin. There is a Rust library called [`WinReg`](https://github.com/gentoo90/winreg-rs) that provides functionality to read/write to the Registry. Editing the Registry is akin to editing someone's `.bashrc`, it's a sin! But... if it means **automatically applying the MSR mod** and **better UX**, then yes I will. The flow would have been:
- User starts XMRig
- Gupax notices XMRig is not admin
- Gupax tells user
- Gupax gives option to AUTOMATICALLY edit registry
- Gupax also gives the option to show how to do it manually

This was the solution I would have gone with, but alas, the abstracted `Command` types I am using to start processes completely ignore this metadata. When Gupax starts XMRig, that `Run as Administrator` flag is completely ignored. Grrr... what options are left?

---

### Windows vs Unix
Unix (macOS/Linux) has a super nice, easy, friendly, not-completely-garbage userland program called: `sudo`. It is so extremely simple to use `sudo` as a sort of wrapper around XMRig since `sudo` isn't completely backwards and actually has valuable flags! No legacy `Administrator`, no UAC prompt, no shells within shells, no low-level system APIs, no messing with the user Registry. 

You get the user's password, you input it to `sudo` with `--stdin` and you execute XMRig with it. Simple, easy, nice. (Don't forget to zero the password memory, though).

With no other option left on Windows, I unfortunately have to fallback to the worst solution: shipping Gupax's binary to have `Administrator` metadata, so that it will automatically prompt users for UAC. This means all child process spawned by Gupax will ALSO have admin rights. Windows having one of the most complicated spaghetti privilege systems is ironically what led me to use the most unsecure option.

Depending on the privilege used, Gupax will error/panic:
- Windows: If not admin, warn the user about potential lower XMRig hashrate
- Unix: IF admin, panic! Don't allow anything. As it should be.

If you're reading this and have a solution (that isn't using Win32), please... please teach me. 
