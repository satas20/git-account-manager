use crate::domain::entity::Profile;
use crate::domain::ports::{AuthProviderPort, SystemIOPort};

/// Add a profile to the system. This is a minimal stub demonstrating the shape
/// of a use case in the hexagon core.
pub fn add_profile<P: AsRef<Profile>, S: SystemIOPort, A: AuthProviderPort>(
    profile: P,
    _storage: &S,
    _auth: &A,
) -> Result<(), String> {
    // In a real implementation this would persist the profile, generate or
    // assign SSH keys, and call the auth provider to register the public key.
    let _profile = profile.as_ref();

    // Return Ok for now as a placeholder.
    Ok(())
}

/// Switch the active profile. Minimal stub for now.
pub fn switch_profile(profile_name: &str, _storage: &dyn SystemIOPort) -> Result<(), String> {
    let _ = profile_name;
    Ok(())
}
