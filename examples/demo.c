const int A = 123;

int add(int a, int b) {
    return a + b;
}

int main() {
    int x;
    x = A;
    if (x > 100) {
        x = x + 1;
    } else {
        x = x - 1;
    }
    return x;
}