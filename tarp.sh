   cargo tarpaulin --skip-clean --all > /tmp/tarpaulin.out
   last=$( tail -n 1 /tmp/tarpaulin.out )
   spl=( $last )
   str=${spl[0]}
   IFS='%';
   read -rasplitIFS<<< "$str"
   cur=${splitIFS[0]}
   timestamp=$(date +%s)
   IFS=' ';
   last_tarpaulin_summary=$( tail -n 1 docs/tarpaulin_summary.txt)
   last_tarpaulin_summary_split=( $last_tarpaulin_summary )
   # only update at most once per hour
   echo $last_tarpaulin_summary;
   limit_l=`expr ${last_tarpaulin_summary_split[0]} + 3600`
   echo "limit=$limit_l,timestamp=$timestamp"
   if [ $limit_l -le $timestamp ]
   then
        echo "updating"
        gnuplot -e "set terminal svg;  plot 'docs/tarpaulin_summary.txt' using 1:2 with lines" > docs/bmw.svg ;
        echo "$timestamp ${splitIFS[0]}" >> docs/tarpaulin_summary.txt
        git add --all
        git commit -m"Pipelines-Bot: Updated site Source Version is $(Build.SourceVersion)";
        git push https://$(github_pat)@github.com/37miners/bmw.git
   else
        echo "not updating too recent"
   fi

