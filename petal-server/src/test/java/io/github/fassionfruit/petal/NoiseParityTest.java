package io.github.fassionfruit.petal;

import net.minecraft.util.RandomSource;
import net.minecraft.world.level.levelgen.synth.Noise;
import net.minecraft.world.level.levelgen.synth.PerlinNoise;
import net.minecraft.world.level.levelgen.synth.SmearedPerlinNoise;

public final class NoiseParityTest {

    private static final long SEED = 123456789L;

    private static final double[][] POSITIONS = {
            {0.0, 0.0, 0.0},
            {1.0, 2.0, 3.0},
            {-1.0, -2.0, -3.0},
            {0.5, 0.25, 0.75},
            {12.5, 64.0, -32.5},
            {-128.25, 80.5, 256.75},
            {100000.0, -100000.0, 100000.0}
    };

    public static void main(String[] args) {
        testPerlin();
        testSmearedPerlin();
    }

    private static void testPerlin() {
        PerlinNoise noise = new PerlinNoise(
                RandomSource.create(SEED)
        );

        dump("PerlinNoise", noise, noise);
    }

    private static void testSmearedPerlin() {
        SmearedPerlinNoise noise = new SmearedPerlinNoise(
                RandomSource.create(SEED),
                0.5D
        );

        dump("SmearedPerlinNoise", noise, noise);
    }

    private static void dump(String name, Noise noise, PerlinNoise perlin) {
        System.out.println();
        System.out.println("=== " + name + " ===");

        StringBuilder config = new StringBuilder();
        perlin.parityConfigString(config);
        System.out.println("CONFIG: " + config);

        for (double[] position : POSITIONS) {
            float value = noise.get(
                    position[0],
                    position[1],
                    position[2]
            );

            System.out.printf(
                    "x=%s y=%s z=%s value=%s bits=%08x%n",
                    position[0],
                    position[1],
                    position[2],
                    value,
                    Float.floatToRawIntBits(value)
            );
        }
    }
}