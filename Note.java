public class Note {
    // the length value of the note
    public byte length; 
    // uses the american system
    // 1 for whole note/semibreve
    // 2 for half note/minim
    // 4 for quater note/crotchet

    // the beat that the note falls on
    public byte beat;
    // can be on beat 1, 2, 3 or 4 but depends on the species

    // the octave value of the note
    public byte octave;
    // middle C is C4
    // generally stay in range of A2 to G5

    // interger to determine the letter of the note
    public byte pitch;
    public char raw_pitch;
    // a = 0, b = 1, c = 2, d = 3, e = 4, f = 5, g = 6
    
    // to indicate if there are sharps or flats
    public byte accidental;
    // 0 = flat, 1 = natural, 2 = sharp

    // raw value of the note stored in a number
    public long raw;
    // calculating the raw value
}
