# Django-style campus portal settings fixture (DO NOT deploy).

DEBUG = True
SECRET_KEY = "campus-insecure-secret-key-change-me"
CORS_ALLOW_ALL_ORIGINS = True
CSRF_COOKIE_SECURE = False

# Payment (never hardcode in real apps)
RAZORPAY_KEY_ID = "rzp_test_EXAMPLEKEY"
RAZORPAY_KEY_SECRET = "razorpay_secret_hardcoded_example"

# Privacy anti-pattern
def log_kyc(aadhaar_number):
    print(f"aadhaar verification for {aadhaar_number}")
