int main() {
    int acc = 0;

    for (int i = 0; i < 10; i++) {
        acc += i;
    }

    for (int i = 0; i < 10; i++) {
        for (int j = 0; j < 10; j++) {
            acc += i * j;
        }
    }

    return 0;
}
