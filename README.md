# Species Counterpoint Checker

This software is designed to take in 2 part counterpoint and evaluate if the music provided follows the rules of counterpoint

## Format for the input file:

The input file should be comprised of three lines. The first line should be the key signature represented by: `<Value> <Accidental> <Minor/Major>`

For the `<Value>` and `<Accidental>` fields see the _pitch_ section. An Uppercase `M` signals a major key and a lowercase `m` signals a minor key.

For the bottom two lines, the top line should be the melody and the bottom line should be the Cantus Firmi.

The input for a single note comprises of three parts: `<length> <beat> <pitch>`

_Length_ is the time value of the note. This is not needed for first species or the Cantus Firmi. It follows the American system:

```
1 = whole note/semibreve
2 = half note/minim
4 = quater note/crotchet

any other inputs are invalid
```
_Beat_ refers to the beat in a bar that the note lands on. This mostly applies to fourth and fifth species counterpoint, so for first to third species this value can be omitted.

_Pitch_ of a note comprises of three parts: `<Value> <Accidental> <Octave>`

The _value_ of a note is the note letter (from A-G) and **must** be written using captial letters.

The _accidental_ of a note refers to if the note has been raised or lowered by a semitone. Lowercase `b` is used to represent a flat, `#` is used to represent a sharp, lowercase `d` can be used to represent a natural if needed. 

The program will work out the key signature, so if a note has an accidental ruled by the key signature, it does not matter if you put the corresponding accidental next to the note. In theory an accidental should last for a whole bar, this applies here as well.

For example if the key signature was F# Major, both `F` and `F#` will be regarded as the value of `F#`

The _octave_ of a note is a number that refers to the octave a note is in. Middle C has an octave number of 4.

It is generally advised that the pitch values stay between A2 and G5, but the program allows input from A0 to C8.

_Rests_ can be inputted with an `R` but only at the beginning of a melodic line. These will automatically default to a minim rest - this value cannot be changed.

Each note should be separated with a coma. Spaces and pipes (`|`) are ignored and can be used for formatting (ie. note spacings and barlines respectively). Here are some examples of input files:

First Species (note the ommited beat and length values):

```
CM
E4, F4, D5, E5
C3, D3, G2, C3
```

Second Species (note the omitted beat values and the use of flats):

```
BbM
R, 2D4, | 2Eb4, 2G4, | 2C4, 2A4, | 1Bb4
B2,     | C3,        | F3,       | B2

```

Fifth Species:

```
TBD
```

Maybe one day I can make the input process easier...

## Usage:

Run the program using: `cat <file.txt> | java Run <species>`

Where `<file.txt>` is the input file and `<species>` is a value from 1-5 depending on the type of counterpoint you wish to check.

## Things that are checked for:
- Parallel and Direct Octaves/5ths
- Consonance in the voices
- The melodic line and skip rules
- Crossed Voices
- Exceptions (ie cambiatas)