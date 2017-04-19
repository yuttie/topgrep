#!/usr/bin/ruby

PID = 129175

while line = ARGF.gets
  line.chomp!
  if line =~ /^top - (.+?) up/
    time = $1
    line = ARGF.gets.chomp until line == ''
    col_names = ARGF.gets.chomp.split(/\s+/).values_at(1..-1)
    while line = ARGF.gets
      line.chomp!
      break if line == ''
      line.strip!
      values = line.split(/\s+/, col_names.size).map {|x| x.strip}
      printf("%s\t)
      if col_names.length != values.length
        p(col_names)
        p(values)
      end
    end
  end
end
