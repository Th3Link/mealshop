plugins {
    id("com.android.application")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

android {
    namespace = "org.mealshop.pocket.dev"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    // Installed, read-only executable code; never execute a copied writable asset.
    packaging {
        jniLibs.useLegacyPackaging = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    defaultConfig {
        applicationId = if (System.getenv("MEALSHOP_ANDROID_VERIFICATION") == "1")
            "org.mealshop.pocket.verification" else "org.mealshop.pocket.dev"
        manifestPlaceholders["mealshopLabel"] = if (System.getenv("MEALSHOP_ANDROID_VERIFICATION") == "1") "Mealshop verification" else "Mealshop"
        // You can update the following values to match your application needs.
        // For more information, see: https://flutter.dev/to/review-gradle-config.
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        // Uses the version code from pubspec.yaml. When using split APKs, 1000 * ABI_VERSION
        // is added automatically by Flutter. (https://developer.android.com/studio/build/configure-apk-splits#configure-APK-versions)
        // You can force using the value of versionCode by specifying the `-P force-version-code-ignoring-abi=true`
        // flag during build.
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    signingConfigs {
        getByName("debug") {
            val retainedKey = System.getenv("MEALSHOP_ANDROID_KEYSTORE")
            if (!retainedKey.isNullOrBlank()) {
                storeFile = file(retainedKey)
                storePassword = System.getenv("MEALSHOP_ANDROID_STORE_PASSWORD") ?: "android"
                keyAlias = System.getenv("MEALSHOP_ANDROID_KEY_ALIAS") ?: "androiddebugkey"
                keyPassword = System.getenv("MEALSHOP_ANDROID_KEY_PASSWORD") ?: "android"
            }
        }
    }
    buildTypes {
        release {
            signingConfig = null // Production release signing is configured separately.
        }
    }
}

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

flutter {
    source = "../.."
}

val verifyDeviceCore by tasks.registering(Exec::class) {
    val platforms = (project.findProperty("target-platform") as? String ?: "android-arm,android-arm64,android-x64").split(",")
    val mapping = mapOf("android-arm" to "armeabi-v7a", "android-arm64" to "arm64-v8a", "android-x64" to "x86_64")
    workingDir(rootProject.projectDir.resolve("../../.."))
    commandLine(listOf("python3", "scripts/native_core.py", "--check") + platforms.flatMap {
        listOf("--abi", mapping[it] ?: throw GradleException("Unsupported Android core platform"))
    })
}
tasks.named("preBuild") { dependsOn(verifyDeviceCore) }

// Local, bundled recognition. No Play Services, external inference or runtime model download.
dependencies {
    implementation(files("../../../../var/mobile/capture-libs/tesseract4android-4.9.0.aar"))
    implementation(files("../../../../var/mobile/capture-libs/zxing-core-3.5.4.jar"))
}
