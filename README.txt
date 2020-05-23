### John McMenemy's Fourth Year Project: Building a Database with Rust ###

A video demonstration was made for this project and can be found here: https://youtu.be/NnFyCwhaCpE

To compile and run the software herein please make sure Rust is installed: https://www.rust-lang.org/
An internet connection is requiered the first time the software is built since external libraries will be
downloaded.

Once that has been done, a command prompt (OS independent) can be opened in the Database Client and Server folders.
To run the server, type in "cargo run". Then, after starting the server, the client can be run using the same procedure.
By default the server and client run on the localhost, on port 6142 (127.0.0.1:6142). If one wishes to change that,
one should type in "cargo run" followed by the address and port e.g. "cargo run 98.60.78.2:5555" for port 5555 at 
adrress 98.60.78.2.

The client is default set to run typed in commands. 
The commands accepted by the server are:
- "get;" to display all base level keys

- "resetlog;" to wipe the log file (please reset the server for the changes to take effect)

- "set <Key> <Value>;" where <Key> and <Value> can be replaced appropriately, to access database folders
please add a slash between folder names e.g. "Users/JohnSmith/Salary" where the last value without a slash is
the key (file) that stores the value (Salary in the example above).

- "get <Key>;" To get the values or list of keys of a folder, <Key> follows the same syntax rules as above

- "remove <Key>;" To remove either a key or a folder, <Key> follows the same syntax rules as above

- "exit;" to shutdown the server

To run one of the automated tests, please change the value
of "test_number" which is found in Database - Client/main.rs
Comments in the file indicate which test number belongs to which test.

To run the bash script test (found in the Database - Server folder) one needs a bash command line.
Before running the test, please make sure that the test_number in the Client is set to 3.
Then, simply type "./serverSleep.sh" into the command line which should be opened in the server folder.

Older work produced in the inital stages of the project, such as the string parser or the concurrent counter modifier
can be found on the "PreviousWork" branch on the university GitLab:
https://gitlab.cis.strath.ac.uk/djb17168/building-a-database-with-rust/-/tree/PreviousWork

There are some Gifs showcasing some of the initial work in the "Demo Gifs" folder. 