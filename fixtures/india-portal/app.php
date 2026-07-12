<?php
// Intentionally vulnerable India-style portal fixture (DO NOT deploy).
// Simulates campus / municipal CMS patterns for Bugbee engines.

$razorpay_key = "rzp_live_EXAMPLEKEY12345";
$razorpay_secret = "pay_secret_DO_NOT_USE_IN_PROD";

function search() {
    $q = $_GET['q'];
    // SQL injection sink
    $sql = "SELECT * FROM students WHERE name = '" . $q . "'";
    mysqli_query($conn, $sql);
}

function run_cmd() {
    $cmd = $_GET['cmd'];
    // command injection
    system($cmd);
}

function load_blob() {
    $data = $_POST['payload'];
    return unserialize($data);
}

// misconfig
ini_set('display_errors', '1');
?>
