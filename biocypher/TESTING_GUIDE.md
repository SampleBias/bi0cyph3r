# Testing Guide for bi0cyph3r

> Historical document: web/Flask/API instructions below describe the retired architecture. See the [current terminal-only README](../README.md) for setup and usage.

## Quick Start Testing

### 1. Install Dependencies (if not already done)

```bash
# Activate virtual environment (if using one)
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt
```

### 2. Start the Application

```bash
python app.py
```

The application will start on `http://127.0.0.1:5000/` (or `http://localhost:5000/`)

### 3. Access the Application

Open your browser and navigate to:
```
http://localhost:5000/
```

**Expected Behavior:** You should be automatically redirected to the dashboard (no login required!)

---

## Testing Checklist

### ✅ Navigation & UI

- [ ] **Landing Page**
  - Visit `http://localhost:5000/`
  - Should redirect directly to dashboard (no login page)
  
- [ ] **Dashboard**
  - Check that all navigation links work (Dashboard, Encode, Decode, Contact)
  - Verify no "Logout" button appears
  - Verify no user info display
  - Check that cards are displayed properly
  - Test responsive design by resizing browser window

- [ ] **Navigation Bar**
  - Click each link: Dashboard, Encode, Decode, Contact
  - Verify active state highlighting works
  - Check that navigation is clean and modern

### ✅ Encoding Functionality

- [ ] **Basic Mode**
  1. Go to Encode page
  2. Enter a test message: `Hello World`
  3. Select "Basic DNA Encoding" mode
  4. Click "Encode to DNA"
  5. Verify DNA sequence appears
  6. Check that stats are displayed (length, base counts, GC content)
  7. Test copy button - should show "✓ Copied!" feedback

- [ ] **Nanopore Mode**
  1. Select "Nanopore Optimized" mode
  2. Toggle "Error correction" checkbox (should appear)
  3. Enter message: `Test nanopore encoding`
  4. Click encode
  5. Verify longer sequence (due to error correction)
  6. Check nanopore-specific stats

- [ ] **Secure Mode**
  1. Select "Secure (AES + DNA)" mode
  2. Password field should appear
  3. Enter a strong password (at least 8 chars, mixed case, numbers, symbols)
  4. Enter message: `Secret message`
  5. Click encode
  6. Verify encrypted DNA sequence appears
  7. Check security info section displays encryption details

- [ ] **Safety Screening**
  1. After encoding any message, click "🛡️ Safety Screen" button
  2. Verify loading state appears
  3. Check safety report displays with:
     - Safety status (SAFE/CAUTION/UNSAFE)
     - Sequence analysis
     - Pathogen risk analysis
     - Natural occurrence check
     - Recommendations

### ✅ Decoding Functionality

- [ ] **Basic Decoding**
  1. Go to Decode page
  2. Copy a DNA sequence from encode results
  3. Paste into decode textarea
  4. Select "Basic DNA Decoding"
  5. Click "Decode to Text"
  6. Verify original message is restored

- [ ] **Nanopore Decoding**
  1. Use a nanopore-encoded sequence
  2. Select "Nanopore Optimized" mode
  3. Enable error correction if it was used during encoding
  4. Verify correct decoding

- [ ] **Secure Decoding**
  1. Use a secure-encoded sequence
  2. Select "Secure (AES + DNA)" mode
  3. Enter the same password used for encoding
  4. Verify message is decrypted correctly
  5. Test with wrong password - should show error

### ✅ UI/UX Testing

- [ ] **Character Counters**
  - Type in message textarea - counter should update
  - Type in sequence textarea - counter should update
  - Check counter formatting (e.g., "42 characters", "128 bases")

- [ ] **Form Validation**
  - Try submitting empty form - should show error
  - Try invalid DNA sequence (non-ATCG characters) - should reject
  - Try very long message - should enforce limits
  - Check error messages are clear and helpful

- [ ] **Copy Functionality**
  - Test copy button on encoded results
  - Test copy button on decoded results
  - Verify success feedback (✓ Copied!)
  - Paste copied text to verify it's correct

- [ ] **Responsive Design**
  - Test on mobile-sized viewport (< 768px)
  - Test on tablet-sized viewport (768px - 1024px)
  - Verify navigation collapses appropriately
  - Check that cards stack properly on mobile
  - Test form layouts on small screens

### ✅ Error Handling

- [ ] **Invalid Input**
  - Try invalid DNA sequence: `ATCGXYZ123`
  - Should show clear error message
  - Try empty message/sequence
  - Should show validation error

- [ ] **API Errors**
  - Test rate limiting (make 10+ requests quickly)
  - Should show appropriate error message
  - Network errors should be handled gracefully

### ✅ Contact Page

- [ ] **Contact Page**
  - Navigate to Contact page
  - Verify all links work
  - Check email links are properly formatted
  - Verify page displays correctly

---

## Quick Test Commands

### Start the Server

```bash
cd /Users/jamesutley/biocypher
python app.py
```

### Test Basic Flow

1. Open browser: `http://localhost:5000/`
2. Should redirect to dashboard automatically
3. Click "Encode Now" → Enter "Hello" → Encode
4. Click "Decode Now" → Paste the sequence → Decode
5. Verify "Hello" is restored

---

## Common Issues & Solutions

### Issue: "Module not found" error
**Solution:** 
```bash
source venv/bin/activate  # or venv\Scripts\activate on Windows
pip install -r requirements.txt
```

### Issue: Port 5000 already in use
**Solution:** Edit `app.py` last line:
```python
app.run(debug=True, port=5001)
```

### Issue: Styles not loading
**Solution:** Clear browser cache (Ctrl+Shift+R / Cmd+Shift+R)

---

## Automated Tests (Optional)

```bash
python test_dna_crypto.py
python test_nanopore_dna_crypto.py
python test_secure_nanopore_dna_crypto.py
```
