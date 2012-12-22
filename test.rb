require 'hiredis'
require 'redis'

redis = Redis.new(:host => '127.0.0.1')
redis.set("abc", "XXX")
10_000.times {
  redis.get("abc")
}
