#!/usr/bin/ruby

def parse(stream)
  Enumerator.new do |y|
    while line = stream.gets
      snapshot = {}

      line.chomp!
      if line =~ /^top - (.+?) up/
        time = $1
        snapshot[:time] = time
        snapshot[:procs] = []

        line = stream.gets.chomp until line == ''

        col_names = stream.gets.chomp.split(/\s+/).values_at(1..-1)
        while line = stream.gets
          line.chomp!
          line.strip!
          break if line == ''

          values = line.split(/\s+/, col_names.size).map {|x| x.strip}
          snapshot[:procs] << Hash[col_names.map {|n| n.delete('^0-9a-zA-Z').downcase.to_sym }.zip(values)]
        end
        y << snapshot
      end
    end
  end
end

PID = 129175

parse(ARGF).each do |s|
  p s[:time]
  p s[:procs][0]
end
