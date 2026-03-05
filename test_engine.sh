#!/bin/bash

BASE_URL="http://127.0.0.1:8080"

echo "---------------------------------"
echo "1️⃣ Initializing market price"
echo "---------------------------------"

curl -s -X POST $BASE_URL/price/update \
-H "Content-Type: application/json" \
-d '{"price":100}'

echo -e "\n"

echo "---------------------------------"
echo "2️⃣ Opening position"
echo "---------------------------------"

OPEN_RESPONSE=$(curl -s -X POST $BASE_URL/position/open \
-H "Content-Type: application/json" \
-d '{
"asset":"SOL",
"margin":100,
"leverage":5,
"position_type":"Long"
}')

echo $OPEN_RESPONSE
echo -e "\n"

POSITION_ID=$(echo $OPEN_RESPONSE | grep -o '"id":"[^"]*"' | cut -d'"' -f4)

echo "Extracted Position ID:"
echo $POSITION_ID
echo -e "\n"

echo "---------------------------------"
echo "3️⃣ Checking open positions"
echo "---------------------------------"

curl -s $BASE_URL/positions
echo -e "\n"

echo "---------------------------------"
echo "4️⃣ Simulating price increase"
echo "---------------------------------"

curl -s -X POST $BASE_URL/price/update \
-H "Content-Type: application/json" \
-d '{"price":120}'

echo -e "\n"

echo "---------------------------------"
echo "5️⃣ Checking positions after price move"
echo "---------------------------------"

curl -s $BASE_URL/positions
echo -e "\n"

echo "---------------------------------"
echo "6️⃣ Closing position"
echo "---------------------------------"

curl -s -X POST $BASE_URL/position/close \
-H "Content-Type: application/json" \
-d "{\"position_id\":\"$POSITION_ID\"}"

echo -e "\n"

echo "---------------------------------"
echo "7️⃣ Final positions state"
echo "---------------------------------"

curl -s $BASE_URL/positions
echo -e "\n"

echo "---------------------------------"
echo "✅ Test Complete"
echo "---------------------------------"