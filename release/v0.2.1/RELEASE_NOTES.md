### Fixed

- Word lookups no longer leave the card empty for half a minute. The translation
  is rendered as soon as the provider answers - 266-679 ms in place of the
  24-28 s a lookup used to take - and the dictionary details follow on their own
  through the new `word_details` command, merging into the card that is already
  on screen instead of holding it back.
- Phonetic symbols, parts of speech, definitions and the example sentence are
  looked up by two sources at once, and the free endpoint that carries the
  example gets a short budget of its own: one source timing out no longer
  discards what the other one found. The dictionary timeout went from 4 s to 8 s,
  which is what the endpoint's variable response time (0.75-20 s) needs.
- The history panel refreshes itself. A translation emits `glossy://history`, so
  the list in the settings window is current without restarting the app, and the
  details that arrive late are written back to the stored entry.
- The history search box follows the shared input rule - same height, radius,
  background and padding as every other field - instead of keeping the browser
  default look.
