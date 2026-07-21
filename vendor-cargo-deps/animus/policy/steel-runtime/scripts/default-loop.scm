(define (animus-decide)
  (if animus-last-model-message
      (animus-respond animus-last-model-message)
      (animus-model animus-summary)))
