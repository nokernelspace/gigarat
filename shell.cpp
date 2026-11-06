#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <termios.h>

struct termios orig_termios;
FILE *bash_pipe = NULL;

void disable_raw_mode() {
    tcsetattr(STDIN_FILENO, TCSAFLUSH, &orig_termios);
    if (bash_pipe) {
        pclose(bash_pipe); // Cleanly close the pipe and wait for bash to exit
    }
}

void enable_raw_mode() {
    tcgetattr(STDIN_FILENO, &orig_termios);
    atexit(disable_raw_mode); 

    struct termios raw = orig_termios;
    // Disable canonical mode, local echo, and signals (Ctrl-C/Z)
    raw.c_lflag &= ~(ICANON | ECHO | ISIG);
    raw.c_cc[VMIN] = 1;
    raw.c_cc[VTIME] = 0;

    tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw);
}

int main() {
    // Open a writable pipe straight to bash
    // "w" means we write to bash's STDIN
    bash_pipe = popen("bash", "w");
    if (!bash_pipe) {
        perror("Failed to start bash pipe");
        return 1;
    }

    enable_raw_mode();
    printf("Shell Active. Commands are piped to bash. Press Ctrl-D (Keycode 4) to exit.\n\r");
    printf("$ ");
    fflush(stdout);

    char c;
    while (read(STDIN_FILENO, &c, 1) == 1) {
        if (c == 4) { // Ctrl-D character code (EOF)
            break;
        } 
        
        else if (c == 127 || c == 8) { // Backspace handling
            // Do basic local terminal erasure, but don't pass backspace blindly to pipe
            printf("\b \b");
            fflush(stdout);
        } 
        
        else if (c == '\r' || c == '\n') { // Enter Key
            // Raw mode registers Enter as '\r'. Translate to newline for bash.
            printf("\n\r");
            fflush(stdout);

            fputc('\n', bash_pipe);
            fflush(bash_pipe); // Flush instantly so bash executes it now
            
            printf("$ ");
            fflush(stdout);
        } 
        
        else { // Normal printable keys
            // 1. Show the user what they typed
            printf("%c", c);
            fflush(stdout);

            // 2. Secretly pass the exact character directly down the pipe to bash
            fputc(c, bash_pipe);
        }
    }

    printf("\n\rExiting cleanly.\n\r");
    return 0;
}

