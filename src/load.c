#include <dlfcn.h>
#include <stdio.h>

typedef void (*greet_t)(const char *name);
typedef int (*add_t)(int a, int b);
// typedef int32_t (*add2_t)(int32_t a, int32_t b);
typedef void (*cus_t)();

int main(void) {
    // this was `./libmain.so`
    void *lib = dlopen("./mid_lib.so", RTLD_LAZY);
    if (!lib) {
        fprintf(stderr, "failed to load library\n");
        return 1;
    }
    void* cus = dlsym(lib, "greet");
    if (!cus) {
        fprintf(stderr, "could not look up symbol 'cus'\n");
        return 1;
    } 
    return 1;
    greet_t greet = (greet_t) dlsym(lib, "_ZN4test5greet17h00995c547cf20126E");
    add_t add = (add_t) dlsym(lib, "add");
    if (!greet) {
        fprintf(stderr, "could not look up symbol 'greet'\n");
        return 1;
    }
    int res = add(12, 25);
    printf("%d\n", res);
    greet("venus");
    dlclose(lib);
    return 0;
}