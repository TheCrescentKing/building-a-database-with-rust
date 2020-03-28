LASTSERVER= tail -1 log.txt | cut -d' ' -f3
cd ../Database\ -\ Client/
PREVOK=0
while IFS= read -r line
do
   echo "line is: $line"
   if (($PREVOK == 1))
   then
     if [[ "$line" =~ .*"$LASTSERVER".* ]]
     then
       echo "Test Passed."
       break
     else
       echo "Test Failed! Ok Received but wrong String!"
       break
     fi
   else
     if [[ "$line" =~ .*"Ok".* ]]
     then
       PREVOK=1
     fi
   fi
done < <(tac output.txt)
