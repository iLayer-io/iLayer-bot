local prefix = "order:"

-- Check if the index already exists
local existing_indices = redis.call("FT._LIST")
for _, idx in ipairs(existing_indices) do
    if idx == index_name then
        return "Index already exists"
    end
end

-- Create the index
redis.call("FT.CREATE", "order_deadline_idx", "ON", "JSON", "PREFIX", "1", prefix, "SCHEMA",
           "$.deadline", "AS", "deadline", "TEXT")

redis.call("FT.CREATE", "order_primary_filler_deadline_idx", "ON", "JSON", "PREFIX", "1", prefix, "SCHEMA",
           "$.primary_filler_deadline", "AS", "deadline", "TEXT")

return "Indexes are created"