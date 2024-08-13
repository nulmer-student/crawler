int many_loops(int a, int b) {
    for (int i; i < 10; i++) {
        a += i;
    }

    for (int i; i < 10; i++) {
        for (int j; j < 10; j++) {
            b += j * i;
        }
    }

    for (int i; i < 10; i++) {
        for (int j; j < 10; j++) {
            b += j * i;
        }

        for (int j; j < 10; j++) {
            a += j * i;
        }
    }

    for (int i; i < 10; i++) {
        for (int j; j < 10; j++) {
            for (int k; k < 10; k++) {
                a += j * i + k;
            }
        }
    }

    return a + b;
}


int bar(int *a, int *b, int n) {
    int c = 0;

    #pragma clang loop scalar_interpolation(enable)
    for (int i = 0; i < n; i++) {
        c += a[i] * b[i];
    }

    return c;
}

int baz(int *a, int *b, int n) {
    int c = 0;

    #pragma clang loop scalar_interpolation(enable)
    for (int i = 0; i < n; i++) {
        c += a[i] + b[i];
    }

    return c;
}

void foo(float *a, float *b, float* c, int n) {
    #pragma clang loop scalar_interpolation(enable)
    for (int i = 0; i < n; i++) {
        c[i] = a[i] + b[i];
    }

    #pragma clang loop scalar_interpolation(enable)
    for (int i = 0; i < n; i++) {
        c[i] += 1.0;
    }
}
