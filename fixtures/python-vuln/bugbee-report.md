# Hardcoded password assignment

**Severity:** Medium  
**CWE:** n/a  
**Rule:** `secrets.generic_password_assign`  
**Finding ID:** `9d1681f6074f018985ed00564f0e58dd`  
**Adjudication:** inconclusive  
**Verified:** yes  
**BRS / ECS:** 50 / 100  

## Description

Hardcoded password assignment detected in source

**NSAE:** noisy symbolic + strong neural → prove

## Reproduction

1. Open `app.py` at line 7.
2. Observe the following code:

```
password = "…***
```

### PoC steps (authorized / local only)

1. Open authorized codebase file: app.py
2. Go to line 7
3. Observe: Hardcoded password assignment detected in source
4. Confirm no compensating control in adjacent lines
5. Document business impact for report

## Impact

Hardcoded credentials can be extracted from source or builds and reused for unauthorized access.

## Fix

Remove secrets from source. Load from a secret manager or environment; rotate exposed credentials.

## Evidence

- **secret_pattern:** secrets.generic_password_assign matched
- **context_window:** lines 4-10:
- **nsae:** symbolic=noisy neural=strong conf=0.98 → inconclusive (noisy symbolic + strong neural → prove)
- **prover:** PASS pattern @ app.py:7


---

# Dangerous eval()

**Severity:** Critical  
**CWE:** CWE-95  
**Rule:** `owasp.python.eval`  
**Finding ID:** `b5ba74b8d108a0d488718df5a8a89242`  
**Adjudication:** vulnerable  
**Verified:** yes  
**BRS / ECS:** 95 / 100  

## Description

Dynamic eval of untrusted input can lead to RCE.

**NSAE:** symbolic strong with neural support

## Reproduction

1. Open `app.py` at line 16.
2. Observe the following code:

```
return eval(expr)
```

### PoC steps (authorized / local only)

1. Open authorized codebase file: app.py
2. Go to line 16
3. Observe: Dynamic eval of untrusted input can lead to RCE.
4. Confirm no compensating control in adjacent lines
5. Document business impact for report

```bash
# Local fixture only — do not run against unauthorized hosts
# curl -s 'http://127.0.0.1:8080/vuln?q=test'
```

## Impact

An attacker who can influence the tainted input may achieve remote code execution or command injection, leading to full host compromise in the worst case.

## Fix

Remove dynamic code execution. Use safe parsers or explicit allowlisted operations.

## Evidence

- **pattern_match:** rule owasp.python.eval matched line 16
- **context_window:** lines 13-19:
- **nsae:** symbolic=strong neural=strong conf=0.99 → vulnerable (symbolic strong with neural support)
- **nsae:** symbolic=strong neural=strong conf=0.99 → vulnerable (symbolic strong with neural support)
- **prover:** PASS pattern @ app.py:16
- **carlini:** iter=0 conf=0.99 adj=vulnerable ver=Confirmed reason=severity=critical; ecs=100; line_is_sink; source_and_sink_present; no_guards_detected; rce_class_rule
- **prover:** PASS pattern @ app.py:16


---

# shell=True in subprocess

**Severity:** High  
**CWE:** CWE-78  
**Rule:** `owasp.command.shell_true`  
**Finding ID:** `ec8aa6bb2b97708ada5c58d1ce46e16b`  
**Adjudication:** vulnerable  
**Verified:** yes  
**BRS / ECS:** 75 / 100  

## Description

subprocess with shell=True is a common command-injection sink.

**NSAE:** symbolic strong with neural support

## Reproduction

1. Open `app.py` at line 11.
2. Observe the following code:

```
subprocess.run(cmd, shell=True)
```

### PoC steps (authorized / local only)

1. Open authorized codebase file: app.py
2. Go to line 11
3. Observe: subprocess with shell=True is a common command-injection sink.
4. Confirm no compensating control in adjacent lines
5. Document business impact for report

```bash
# Local fixture only — do not run against unauthorized hosts
# curl -s 'http://127.0.0.1:8080/vuln?q=test'
```

## Impact

An attacker who can influence the tainted input may achieve remote code execution or command injection, leading to full host compromise in the worst case.

## Fix

Avoid shell=True. Pass argument vectors to subprocess without a shell; validate inputs.

## Evidence

- **pattern_match:** rule owasp.command.shell_true matched line 11
- **context_window:** lines 8-14:
- **nsae:** symbolic=strong neural=strong conf=0.99 → vulnerable (symbolic strong with neural support)
- **nsae:** symbolic=strong neural=strong conf=0.99 → vulnerable (symbolic strong with neural support)
- **prover:** PASS pattern @ app.py:11
- **carlini:** iter=0 conf=0.99 adj=vulnerable ver=Confirmed reason=severity=high; ecs=100; line_is_sink; source_and_sink_present; no_guards_detected; rce_class_rule
- **prover:** PASS pattern @ app.py:11

