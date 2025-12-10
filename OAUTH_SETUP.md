# OAuth Setup Guide

This guide shows you how to set up OAuth credentials for Git Account Manager, both for **development** and for **embedded credentials in releases**.

---

## 🎯 Two Ways to Use OAuth

### **Option 1: Embedded Credentials (Recommended for Users)**
The released binaries come with OAuth credentials **already embedded**. Users just download and run - no setup needed!

### **Option 2: Custom Credentials (For Development/Custom Builds)**
Developers can use their own OAuth apps for testing or custom deployments.

---

## 🔧 For Repository Maintainers: Setting Up GitHub Secrets

To embed OAuth credentials in the released binaries, you need to add them as **GitHub Repository Secrets**.

### **Step 1: Create OAuth Applications**

#### **GitHub OAuth App**
1. Go to [GitHub Developer Settings](https://github.com/settings/developers)
2. Click **"New OAuth App"**
3. Fill in:
   - **Application name**: `Git Account Manager`
   - **Homepage URL**: `https://github.com/satas20/git-account-manager`
   - **Authorization callback URL**: `http://127.0.0.1:8787/callback`
4. Click **"Register application"**
5. Copy the **Client ID** and **Client Secret**

#### **GitLab OAuth App**
1. Go to [GitLab Applications](https://gitlab.com/-/profile/applications)
2. Click **"Add new application"**
3. Fill in:
   - **Name**: `Git Account Manager`
   - **Redirect URI**: `http://127.0.0.1:8788/callback`
   - **Scopes**: Select `api` and `read_user`
4. Click **"Save application"**
5. Copy the **Application ID** and **Secret**

---

### **Step 2: Add Secrets to GitHub Repository**

1. Go to your repository on GitHub
2. Click **Settings** → **Secrets and variables** → **Actions**
3. Click **"New repository secret"**
4. Add these **4 secrets**:

| Secret Name | Value | Description |
|-------------|-------|-------------|
| `OAUTH_GITHUB_CLIENT_ID` | Your GitHub Client ID | From step 1 |
| `OAUTH_GITHUB_CLIENT_SECRET` | Your GitHub Client Secret | From step 1 |
| `OAUTH_GITLAB_APP_ID` | Your GitLab Application ID | From step 1 |
| `OAUTH_GITLAB_CLIENT_SECRET` | Your GitLab Secret | From step 1 |

**Important:**
- Use the prefix `OAUTH_` to avoid confusion with runtime env vars
- Keep these secrets secure - never commit them to git
- These will be embedded in the binary during GitHub Actions builds

---

### **Step 3: Verify It Works**

Once secrets are added:

1. Create a new release:
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```

2. GitHub Actions will build binaries with **embedded credentials**

3. Download and test:
   ```bash
   # Download the binary
   curl -L https://github.com/satas20/git-account-manager/releases/latest/download/git-acc-mngr-linux-x86_64.tar.gz -o binary.tar.gz
   tar -xzf binary.tar.gz

   # Run it - should work without setting env vars!
   ./git-acc-mngr-linux-x86_64
   ```

4. If OAuth fails, check the GitHub Actions build logs

---

## 👨‍💻 For Developers: Local Development

### **Option A: Use Your Own OAuth Apps**

Create your own OAuth apps (same steps as above) and set environment variables:

```bash
# Create .env file
cat > .env << 'EOF'
GITHUB_CLIENT_ID=your_github_client_id
GITHUB_CLIENT_SECRET=your_github_client_secret
GITLAB_APP_ID=your_gitlab_app_id
GITLAB_CLIENT_SECRET=your_gitlab_secret
EOF

# Load environment variables
source .env

# Run the app
cargo run --release
```

**Pros:**
- ✅ No rate limits from shared credentials
- ✅ Full control over OAuth scopes
- ✅ Can test OAuth flow changes

**Cons:**
- ⚠️ Requires manual setup
- ⚠️ Each developer needs their own apps

---

### **Option B: Build with Embedded Credentials**

Build locally with credentials embedded:

```bash
# Set build-time environment variables
export GITHUB_CLIENT_ID="your_id"
export GITHUB_CLIENT_SECRET="your_secret"
export GITLAB_APP_ID="your_app_id"
export GITLAB_CLIENT_SECRET="your_secret"

# Build - credentials will be embedded
cargo build --release

# Run - no env vars needed!
./target/release/git-acc-mngr
```

---

## 🔍 How It Works

The code uses a **fallback mechanism**:

```rust
// Try runtime env first, then compile-time env
let client_id = env::var("GITHUB_CLIENT_ID")
    .or_else(|_| option_env!("GITHUB_CLIENT_ID").map(String::from).ok_or(()))
    .map_err(|_| "Missing GITHUB_CLIENT_ID")?;
```

**Priority order:**
1. **Runtime environment variable** (set when running the binary)
2. **Compile-time environment variable** (embedded during build)
3. **Error** if neither is available

This means:
- ✅ Users can override embedded credentials if needed
- ✅ Developers can test without embedding
- ✅ Released binaries work out-of-the-box

---

## 🔒 Security Considerations

### **For Repository Maintainers:**

**DO:**
- ✅ Use GitHub Secrets for OAuth credentials
- ✅ Limit OAuth scopes to minimum required
- ✅ Monitor GitHub/GitLab for unusual API usage
- ✅ Rotate credentials if they're compromised

**DON'T:**
- ❌ Commit OAuth credentials to git
- ❌ Share credentials publicly
- ❌ Use personal OAuth apps for production releases

### **For Users:**

The embedded credentials are **safe** because:
- They only allow OAuth flow (not direct account access)
- Users must manually authorize the app in their browser
- Tokens are encrypted locally on your machine
- You can use your own credentials if you prefer

### **For Developers:**

If you're distributing custom builds:
- Create your own OAuth apps
- Use repository secrets if using CI/CD
- Document how users can override with their own credentials

---

## 📊 OAuth App Configuration Summary

### **GitHub OAuth App Settings**

```yaml
Application name: Git Account Manager
Homepage URL: https://github.com/satas20/git-account-manager
Authorization callback URL: http://127.0.0.1:8787/callback
Scopes: read:user, user:email, write:public_key (automatically requested)
```

### **GitLab OAuth App Settings**

```yaml
Name: Git Account Manager
Redirect URI: http://127.0.0.1:8788/callback
Scopes: api, read_user
Confidential: Yes (default)
```

---

## 🐛 Troubleshooting

### **Error: "Missing GITHUB_CLIENT_ID"**

**If using released binary:**
- The binary wasn't built with embedded credentials
- Set environment variables manually (see Option A above)

**If building from source:**
- Set environment variables before building
- Or set them at runtime

### **Error: "OAuth callback failed"**

**Check:**
- Callback URL matches OAuth app settings exactly
- Ports 8787 (GitHub) and 8788 (GitLab) are not in use
- No firewall blocking localhost connections

### **GitHub Actions build fails with OAuth errors**

**Check:**
- All 4 secrets are set in repository settings
- Secret names match exactly (case-sensitive)
- Secrets have valid values (no quotes, no whitespace)

---

## 📝 Verification Checklist

Before releasing, verify:

- [ ] GitHub OAuth app created and credentials saved
- [ ] GitLab OAuth app created and credentials saved
- [ ] All 4 secrets added to GitHub repository
- [ ] Test build passes in GitHub Actions
- [ ] Downloaded binary works without env vars
- [ ] OAuth flow completes successfully for both providers
- [ ] Credentials not committed to git (check with `git log -p`)

---

## 🔗 References

- [GitHub OAuth Apps Documentation](https://docs.github.com/en/developers/apps/building-oauth-apps)
- [GitLab OAuth Documentation](https://docs.gitlab.com/ee/integration/oauth_provider.html)
- [GitHub Actions Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [Rust `option_env!` macro](https://doc.rust-lang.org/std/macro.option_env.html)
