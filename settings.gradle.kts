import java.util.Locale

pluginManagement {
    repositories {
        gradlePluginPortal()
        maven("https://repo.papermc.io/repository/maven-public/")
    }
}

plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0"
}

if (!file(".git").exists()) {
    val errorText = """
        
        =====================[ ERROR ]=====================
         The Petal project directory is not a properly cloned Git repository.
         
         In order to build Petal from source you must clone
         the Petal repository using Git, not download a code
         zip from GitHub.
         
         Built Petal jars are available for download at
         https://github.com/fassionfruit/petalmc
         
         See https://github.com/petalMC/petal/blob/HEAD/CONTRIBUTING.md
         for further information on building and modifying Petal.
        ===================================================
    """.trimIndent()
    error(errorText)
}

rootProject.name = "petal"
for (name in listOf("petal-api", "petal-server", "petal-checkstyle")) {
    val projName = name.lowercase(Locale.ENGLISH)
    include(projName)
    findProject(":$projName")!!.projectDir = file(name)
}

optionalInclude("test-plugin")

fun optionalInclude(name: String, op: (ProjectDescriptor.() -> Unit)? = null) {
    val settingsFile = file("$name.settings.gradle.kts")
    if (settingsFile.exists()) {
        apply(from = settingsFile)
        findProject(":$name")?.let { op?.invoke(it) }
    } else {
        settingsFile.writeText(
            """
            // Uncomment to enable the '$name' project
            // include(":$name")

            """.trimIndent()
        )
    }
}

gradle.lifecycle.beforeProject {
    val mcVersion = providers.gradleProperty("mcVersion").get().trim()
    val petalChannel = providers.gradleProperty("channel").get().trim()
    val petalBuildNumber = providers.environmentVariable("BUILD_NUMBER").orNull?.trim()?.toInt()
    val versionString = if (petalBuildNumber == null) {
        "$mcVersion.local-SNAPSHOT"
    } else {
        "$mcVersion.build.$petalBuildNumber-${petalChannel.lowercase()}"
    }
    version = versionString
}
