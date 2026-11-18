// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#include <fcntl.h>
#include <stdio.h>
#include <sys/ioctl.h>
#include <unistd.h>

int main(void)
{
    setsid();
    int terminal = open("/dev/ttyS0", O_RDWR);
    if (terminal >= 0) {
        ioctl(terminal, TIOCSCTTY, 1);
        dup2(terminal, 0);
        dup2(terminal, 1);
        dup2(terminal, 2);
        if (terminal > 2) {
            close(terminal);
        }
    }
    puts("cheri-vm init");
    fflush(stdout);
    execl("/bin/sh", "sh", (char *)0);
    perror("/bin/sh");
    for (;;) {
        pause();
    }
}
