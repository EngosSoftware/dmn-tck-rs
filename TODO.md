# TO-DO list for version 0.0.2

## Command-line arguments
 
Add single command-line argument to the application.
This argument is a name of the directory where tests are stored.
The structure of this directory is defined exactly like in 
https://github.com/dmn-tck/tck/tree/master/TestCases.
Before the runner starts, there must be a check if the given
argument is a directory and this directory exists in file system.
If specified argument is not a directory or does not exist,
the proper error message should be printed to standard output. 
