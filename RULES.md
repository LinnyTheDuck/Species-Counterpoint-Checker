# Rule catalogue (draft)

Two voices: a cantus firmus (CF) in semibreves and one counterpoint line (CP), above or below. Major and minor keys only.

Each rule has a stable id the engine and UI share. Severity is `error` (breaks the rule) or `warning` (style, avoid where possible).

Source column:
- `notes` = course notes (MUSI119 species sheets).
- `std` = standard rule from the Fux tradition, written from general knowledge.
- `decision` = a call made where the notes are silent, recorded under Decisions below.
- `assignment` = changed to match a full-marks assignment, recorded under Decisions below.

Interval names are between the two voices unless the rule says melodic. Compound intervals count as their simple form (a 12th is a 5th).

## All species

| id | Rule | Severity | Source |
|---|---|---|---|
| `start-perfect` | First sounding interval is P1, P5 or P8 (CP above); P1 or P8 (CP below). | error | std |
| `end-perfect` | Last interval is P1 or P8. | error | std |
| `cadence-step` | Both voices approach the final note by step, in contrary motion. | error | std |
| `cadence-leading-note` | Penultimate CP note is the leading note (raised 7th in minor) when the CP has the 7–8 motion. | error | std |
| `diatonic` | Only notes of the key, plus raised 6th and 7th in minor. | error | std |
| `minor-6-7` | In minor, raised 6th and 7th go up (to 7 and 8); natural 6th and 7th go down. | warning | std |
| `melodic-forbidden` | No melodic augmented 4ths, diminished 5ths, 7ths or 9ths. | error | notes |
| `melodic-augmented` | No other augmented or diminished melodic intervals. | error | std |
| `melodic-over-octave` | No melodic leap larger than an octave. | error | std |
| `melodic-sixth` | Melodic 6th only as a minor 6th, rising or falling. | warning | std, assignment |
| `leap-recovery` | A leap larger than a 4th is followed by a step in the opposite direction. | warning | std |
| `consecutive-leaps` | No more than two leaps in a row in the same direction; two in a row outline a consonant triad. | warning | std |
| `tritone-outline` | Line does not outline a tritone between its local high and low points. | warning | std |
| `single-climax` | CP has one highest note, used once. | warning | std |
| `range` | CP stays within a 10th. | warning | std |
| `voice-crossing` | CP does not cross the CF. | warning | std, decision |
| `parallel-perfect` | No parallel 5ths, 8ves or unisons between the beats the species rules below say to compare. | error | notes |
| `direct-perfect` | No direct (hidden) 5ths or 8ves: both voices moving the same way into a perfect interval. Error when the upper voice leaps; warning when it moves by step. | error / warning | notes (defined), decision |

## 1st species (semibreve against semibreve)

| id | Rule | Severity | Source |
|---|---|---|---|
| `s1-consonant` | Every interval is P1, m3, M3, P5, m6, M6 or P8 (or compound). P4, 7ths, 9ths and augmented or diminished intervals are not allowed. | error | notes |
| `s1-unison-middle` | Unison only on the first or last note. | error | std |
| `s1-parallels` | `parallel-perfect` and `direct-perfect` apply between every pair of consecutive notes. | error | notes |
| `s1-imperfect-run` | No more than three 3rds or 6ths in a row. | warning | std |
| `s1-repeated-note` | Avoid repeating a CP note, unless it is tied over the barline. | warning | std, assignment |
| `s1-cadence` | Penultimate interval is M6 to P8 (CP above) or m3 to P8 (CP below), compounds allowed. | error | std, decision |

## 2nd species (two minims against a semibreve)

| id | Rule | Severity | Source |
|---|---|---|---|
| `s2-rhythm` | Two minims a bar, last bar a semibreve. May start with a minim rest. | error | std, README |
| `s2-strong-consonant` | Beat 1 is consonant (1st species list). | error | notes |
| `s2-passing-dissonance` | A dissonant beat 2 is a passing note: approached and left by step in the same direction. | error | notes |
| `s2-parallels-strong` | No parallel 5ths or 8ves on consecutive beat 1s. | error | notes |
| `s2-parallels-weak-strong` | No parallel 5ths or 8ves from beat 2 to the next beat 1. | error | notes |
| `s2-direct-octave` | No direct octaves from beat 2 to the next beat 1. | error | notes |
| `s2-direct-fifth` | Avoid direct 5ths from beat 2 to the next beat 1. | warning | decision |
| `s2-unison-strong` | No unison on beat 1 except the first and last bar. | error | std |
| `s2-repeated-note` | No repeated notes. | error | std |
| `s2-cadence` | Penultimate bar ends 5–M6 to P8 above, or equivalent below. | warning | std |

## 3rd species (four crotchets against a semibreve)

| id | Rule | Severity | Source |
|---|---|---|---|
| `s3-rhythm` | Four crotchets a bar, last bar a semibreve. May start with a crotchet rest. | error | std |
| `s3-strong-consonant` | Beat 1 is consonant. | error | std |
| `s3-dissonance` | Dissonance on beats 2–4 only as a passing note or neighbour note (stepwise in and out). | error | std |
| `s3-cambiata` | Exception: nota cambiata (step down to a dissonance, leap down a 3rd, step up) and double neighbour are allowed. | n/a | std, README |
| `s3-parallels` | No parallel 5ths or 8ves on consecutive beat 1s, or from beat 4 to the next beat 1. | error | std |
| `s3-repeated-note` | No repeated notes. | error | std |

## 4th species (syncopation)

| id | Rule | Severity | Source |
|---|---|---|---|
| `s4-rhythm` | Minims tied from beat 2 over the barline; starts with a minim rest; last bar a semibreve. | error | std |
| `s4-weak-consonant` | Beat 2 (start of each tie) is consonant. | error | std |
| `s4-suspension-resolve` | A dissonant beat 1 is a suspension: it resolves down by step to a consonance on beat 2. | error | std |
| `s4-allowed-suspensions` | CP above: 7–6, 4–3, 9–8. CP below: 2–3. Others (for example 2–1 below) are not allowed. | error | std |
| `s4-9-8-limit` | Use 9–8 sparingly; it can sound like octaves. | warning | std |
| `s4-parallels` | No parallel 5ths or 8ves between consecutive beat 2s. | error | std |
| `s4-break` | Breaking the syncopation (untied minims) is allowed, but not for long. | warning | std |
| `s4-cadence` | Cadence is 7–6–8 (CP above) or 2–3–1 (CP below). | error | std |

## 5th species (florid)

| id | Rule | Severity | Source |
|---|---|---|---|
| `s5-mixed` | Each rhythmic figure follows the rules of the species it comes from (minims as 2nd, crotchets as 3rd, ties as 4th). | error | std |
| `s5-quaver-pair` | At most one pair of quavers a bar, only in pairs. | error | notes |
| `s5-quaver-step` | Quavers move by step. | error | notes |
| `s5-quaver-beat` | Quavers only on beat 2 or 4. | error | notes |
| `s5-decorated-suspension` | A suspension may be decorated before its resolution with one of: passing to the lower neighbour of the resolution, passing to the upper neighbour of the suspension, lower neighbour of the resolution, upper neighbour of the suspension, anticipation of the resolution, downward leap to a consonance. | n/a | notes |
| `s5-variety` | Do not stay in one species' rhythm for too long. | warning | std |
| `s5-end` | Last bar a semibreve. | error | std |

## Decisions

I don't have all my course notes, so these follow common Fux-tradition practice. I have used my full marks assignments as tests to confirm or deny rules.

- Direct 5ths and 8ves (`direct-perfect`): the notes define them but do not ban them. Strict two-part writing avoids them, and the step-in-the-upper-voice exception comes from four-part harmony. So: error when the upper voice leaps, warning when it steps.
- 2nd species direct 5ths (`s2-direct-fifth`): the notes only ban direct octaves from beat 2 to beat 1. Direct 5ths there are a warning, not an error.
- Minor keys (`diatonic`, `minor-6-7`, `cadence-leading-note`): raised 6th and 7th are allowed in both voices. The leading note is required at the cadence in whichever voice has the 7–8 motion. Raised 6th and 7th rise, natural 6th and 7th fall.
- CP below the CF (`start-perfect`, `s1-cadence`): starts on P1 or P8 only, since a 5th below puts the CP in the wrong mode. The CF ends 2–1, so the CP below ends 7–1, a m3 (or m10) to P8.
- Voice crossing (`voice-crossing`): texts differ on brief crossing, so it is a warning.
- Spacing: the `std` rule "voices no more than a 10th apart" is dropped. A full-marks first species assignment (A minor, CP below) keeps the voices two octaves apart throughout.
- Ties in 1st species (`s1-repeated-note`): the same assignment ties a repeated note (bars 4–5), which was allowed. A tied repeat is not flagged, and the melody rules treat tied notes as one note.
- Melodic 6ths (`melodic-sixth`): the same assignment falls a minor 6th (E3 to G#2), so falling minor 6ths are allowed. Major 6ths still warn.
