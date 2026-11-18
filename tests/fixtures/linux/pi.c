// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#include <math.h>
#include <stdio.h>

static double arctangent_of_inverse(double inverse)
{
    double power = 1.0 / inverse;
    double sum = 0.0;
    for (int term = 0; term < 30; term++) {
        double part = power / (double)(2 * term + 1);
        sum += (term % 2 == 0) ? part : -part;
        power /= inverse * inverse;
    }
    return sum;
}

int main(void)
{
    volatile double two = 2.0;
    printf("pi %.12f\n", 16.0 * arctangent_of_inverse(5.0) - 4.0 * arctangent_of_inverse(239.0));
    printf("root %.12f\n", sqrt(two));
    return 0;
}
