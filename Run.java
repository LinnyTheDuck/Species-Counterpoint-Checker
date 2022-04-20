// This program is a checker for first to fifth species counterpoint

import java.io.*;

public class Run {
    public static void main(String[] args) {
        
        if (args[0] == null){
            System.out.println("Usage: cat <file.txt> | java Run <species>");
            System.out.println("Options for species include the numbers from and including 1-5");
            return;
        }
        else{
        BufferedReader buf = new BufferedReader(new InputStreamReader(System.in)); // read in file, file should be formatted appropriately

        // taking in input and setting up the arrays
            try {
                //String
                String top = buf.readLine();
                String[] melody = top.split(",");

            } catch (Exception e) {

                e.printStackTrace(); // debugging
            }
        }

    }

    public static boolean first_species(){
        return false;
    }
}
