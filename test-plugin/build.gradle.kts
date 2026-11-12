plugins {
    kotlin("jvm") version "2.4.21"
}

group = "io.github.fassionfruit"
version = ""

repositories {
    mavenCentral()
}

dependencies {
    testImplementation(kotlin("test"))
}

kotlin {
    jvmToolchain(25)
}

tasks.test {
    useJUnitPlatform()
}