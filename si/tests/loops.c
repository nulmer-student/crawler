int foo(int a, int b) {

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

	return a + b;
}
