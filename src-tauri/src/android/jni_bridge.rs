#[cfg(target_os = "android")]
use jni::objects::{JObject, JString, JValue};
#[cfg(target_os = "android")]
use jni::sys::jint;
#[cfg(target_os = "android")]
use jni::JNIEnv;

const MINECRAFT_PACKAGE: &str = "com.mojang.minecraftpe";

#[derive(Debug, thiserror::Error)]
pub enum JniError {
    #[error("JNI error: {0}")] Jni(#[from] jni::errors::Error),
    #[error("Minecraft Bedrock is not installed")] NotInstalled,
}

#[cfg(target_os = "android")]
fn with_env_and_activity<F, R>(f: F) -> Result<R, JniError>
where F: FnOnce(&mut JNIEnv, &JObject) -> Result<R, JniError> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let mut env = vm.attach_current_thread()?;
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };
    f(&mut env, &activity)
}

#[cfg(target_os = "android")]
pub fn is_minecraft_installed() -> Result<bool, JniError> {
    with_env_and_activity(|env, activity| {
        let pm = env.call_method(activity, "getPackageManager", "()Landroid/content/pm/PackageManager;", &[])?.l()?;
        let pkg = env.new_string(MINECRAFT_PACKAGE)?;
        match env.call_method(&pm, "getPackageInfo", "(Ljava/lang/String;I)Landroid/content/pm/PackageInfo;", &[(&pkg).into(), JValue::Int(0)]) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    })
}

#[cfg(target_os = "android")]
pub fn get_minecraft_version() -> Result<String, JniError> {
    with_env_and_activity(|env, activity| {
        let pm = env.call_method(activity, "getPackageManager", "()Landroid/content/pm/PackageManager;", &[])?.l()?;
        let pkg = env.new_string(MINECRAFT_PACKAGE)?;
        let info = env.call_method(&pm, "getPackageInfo", "(Ljava/lang/String;I)Landroid/content/pm/PackageInfo;", &[(&pkg).into(), JValue::Int(0)])?.l()?;
        let vn = env.get_field(&info, "versionName", "Ljava/lang/String;")?.l()?;
        Ok(env.get_string(&JString::from(vn))?.into())
    })
}

#[cfg(target_os = "android")]
pub fn launch_minecraft() -> Result<(), JniError> {
    with_env_and_activity(|env, activity| {
        let pm = env.call_method(activity, "getPackageManager", "()Landroid/content/pm/PackageManager;", &[])?.l()?;
        let pkg = env.new_string(MINECRAFT_PACKAGE)?;
        let intent = env.call_method(&pm, "getLaunchIntentForPackage", "(Ljava/lang/String;)Landroid/content/Intent;", &[(&pkg).into()])?.l()?;
        if intent.is_null() { return Err(JniError::NotInstalled); }
        env.call_method(&intent, "addFlags", "(I)Landroid/content/Intent;", &[JValue::Int(0x10000000 as jint)])?;
        env.call_method(activity, "startActivity", "(Landroid/content/Intent;)V", &[(&intent).into()])?;
        Ok(())
    })
}

#[cfg(not(target_os = "android"))]
pub fn is_minecraft_installed() -> Result<bool, JniError> { Ok(false) }
#[cfg(not(target_os = "android"))]
pub fn get_minecraft_version() -> Result<String, JniError> { Ok("0.0.0".to_string()) }
#[cfg(not(target_os = "android"))]
pub fn launch_minecraft() -> Result<(), JniError> { Err(JniError::NotInstalled) }
