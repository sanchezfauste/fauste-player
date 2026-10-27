// The Android shell of Fauste Player (mobile spec §3). The Rust library
// (libfp_mobile.so) is built into src/main/jniLibs by
// scripts/package/android.sh before Gradle runs.
plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.sanchezfauste.fausteplayer"
    compileSdk = 36
    defaultConfig {
        applicationId = "com.sanchezfauste.FaustePlayer"
        minSdk = 29
        targetSdk = 36
        // From the workspace version (scripts/package/android.sh).
        versionCode = (project.findProperty("versionCode") as String?)?.toInt() ?: 1
        // The workspace version, passed by scripts/package/android.sh.
        versionName = (project.findProperty("versionName") as String?) ?: "0.0.0-dev"
        ndk { abiFilters += listOf("arm64-v8a", "x86_64") }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    // The release key comes from the environment (the release workflow
    // decodes it from its secrets; docs/technical/release-process.md).
    // Without it release builds are signed with the debug key, so they
    // install for audio checks (mobile spec §11) but are never published.
    val releaseKey = listOf(
        "FAUSTE_ANDROID_KEYSTORE",
        "FAUSTE_ANDROID_KEYSTORE_PASSWORD",
        "FAUSTE_ANDROID_KEY_ALIAS",
        "FAUSTE_ANDROID_KEY_PASSWORD",
    ).map { System.getenv(it).orEmpty() }
    signingConfigs {
        if (releaseKey.all { it.isNotEmpty() }) {
            create("release") {
                storeFile = file(releaseKey[0])
                storePassword = releaseKey[1]
                keyAlias = releaseKey[2]
                keyPassword = releaseKey[3]
            }
        }
    }
    buildTypes {
        release {
            signingConfig = signingConfigs.findByName("release")
                ?: signingConfigs.getByName("debug").also {
                    if (gradle.startParameter.taskNames.any { it.contains("Release") }) {
                        logger.warn("No release key: the release build is signed with the debug key")
                    }
                }
        }
    }
    packaging { jniLibs { useLegacyPackaging = false } }
}

kotlin { compilerOptions { jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17) } }

dependencies {
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.core:core-ktx:1.17.0")
    implementation("androidx.games:games-activity:4.4.0")
}
