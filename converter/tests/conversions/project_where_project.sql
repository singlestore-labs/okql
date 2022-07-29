SELECT max(a, b) + 2.5 + 1 * 2 as c, b
FROM (
    SELECT a, 2 as b
    FROM foobar
    WHERE (2 > a)
)
