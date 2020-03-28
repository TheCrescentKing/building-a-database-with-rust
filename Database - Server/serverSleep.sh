echo 'Running Script'
for (( i=1; i <= 5; i++ ))
do
  rm -f log.txt
  (cargo run 127.0.0.1:6183) &
  SPID=$!
  cd ../Database\ -\ Client/
  cargo run 127.0.0.1:6183 &
  sleep $(( ( RANDOM % 6 )  + 1 ))
  kill -9 $SPID
  cd ../Database\ -\ Server/
  RESULT="$(./checkLines.sh)"
  echo $RESULT
  if [[ "$RESULT" =~ .*"Failed".* ]]
  then
    break
  fi
done
