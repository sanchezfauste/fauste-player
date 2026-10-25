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
        versionCode = 1
        // The workspace version, passed by scripts/package/android.sh.
        versionName = (project.findProperty("versionName") as String?) ?: "0.0.0-dev"
        ndk { abiFilters += listOf("arm64-v8a", "x86_64") }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    // Until the release plan brings a release key, release builds are
    // signed with the debug key so they install for audio checks (only
    // release builds are judged for dropouts, mobile spec §11).
    buildTypes {
        release { signingConfig = signingConfigs.getByName("debug") }
    }
    packaging { jniLibs { useLegacyPackaging = false } }
}

kotlin { compilerOptions { jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17) } }

dependencies {
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.core:core-ktx:1.17.0")
    implementation("androidx.games:games-activity:4.4.0")
}
