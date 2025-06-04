---
name: Bug report
about: Create a report to help us improve
title: ''
labels: ''
assignees: ''

---

# [BUG] <brief description>

<!-- Brief summary of the issue -->

Oneil doesn't find parameter `x` when it is defined.

## Relevant Code

<!-- Include any relevant code here -->

### `my_module.on`

```oneil
use my_other_module as m

Value of X: x = 10
```


### `my_other_module.on`

```oneil
Value of Y: y = 15
```


## What did I expect to happen?

<!-- Tell us what you thought the code should do -->

I expected the parameter `x` to be assigned the value of `10`.


## What actually happened?

<!-- Tell us what happened instead, including any error messages or weird behavior -->

Oneil gave the following error when I loaded the file:

```
IDError in model my_module (ID: x): Parameter ID "x" not found in path (my_module).
```


## How can we reproduce the issue?

<!-- Tell us what you did to run the code (e.g., command-line steps, input files, etc.) -->

1. Ran `oneil my_module.on`.
2. Received the following output:
   
   ```
   Oneil 0.13
   Type 'help' for a list of commands or see the README for more information.
   --------------------------------------------------------------------------------
   Loading model my_module.on...
   IDError in model my_module (ID: x): Parameter ID "x" not found in path (my_module).
   ```

## Version or Commit

<!-- What version of the language or interpreter were you using? (If using a
     specific commit, run `git rev-parse --short HEAD` in the `oneil` repository
     and paste the output here) -->

`v0.12` (version) or `a1b2c3d4` (commit)


## Other details

<!-- Optional: anything else that might help? (OS, context, ideas, notes) -->


**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Expected behavior**
A clear and concise description of what you expected to happen.

**Screenshots**
If applicable, add screenshots to help explain your problem.

**Desktop (please complete the following information):**
 - OS: [e.g. iOS]
 - Browser [e.g. chrome, safari]
 - Version [e.g. 22]

**Smartphone (please complete the following information):**
 - Device: [e.g. iPhone6]
 - OS: [e.g. iOS8.1]
 - Browser [e.g. stock browser, safari]
 - Version [e.g. 22]

**Additional context**
Add any other context about the problem here.
