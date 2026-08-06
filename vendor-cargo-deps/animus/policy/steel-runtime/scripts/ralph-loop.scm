(define ralph-completion-promise "COMPLETE")

(define ralph-loop-instructions
  "Goal: complete one unfinished Ralph story per iteration, validate it, persist progress, and output COMPLETE only when every story passes.\n\nSuccess criteria:\n- Select the highest-priority unfinished story from the supplied task state.\n- Make only focused changes for that story.\n- Run the smallest relevant validation checks available through granted host capabilities.\n- Record durable progress and reusable learnings through granted host capabilities.\n- Output COMPLETE only when all stories pass; otherwise end normally so the loop can continue.\n\nConstraints:\n- Treat prd.json, progress.txt, AGENTS.md, branches, commits, shell commands, browser checks, network, credentials, and provider sessions as host-owned capabilities, not ambient authority.\n- Do not open, create, merge, or manage pull requests.\n- Keep evidence safe: report summaries, labels, and hashes instead of raw secrets or oversized payloads.\n\nTask state:\n")

(define (ralph-complete? message)
  (and message (string=? message ralph-completion-promise)))

(define (ralph-followup-context)
  (if animus-last-model-message
      (string-append "\nPrevious iteration summary:\n" animus-last-model-message "\n")
      ""))

(define (ralph-prompt)
  (string-append ralph-loop-instructions animus-summary (ralph-followup-context)))

(define (animus-decide)
  (if (ralph-complete? animus-last-model-message)
      (animus-completion-candidate ralph-completion-promise)
      (animus-model (ralph-prompt))))
