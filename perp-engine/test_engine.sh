#!/bin/bash

BASE_URL="http://127.0.0.1:8080"

echo ""
echo "Testing Perp Engine API"
echo ""

echo "---------------------------------"
echo "1️⃣ Opening position"
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
echo "2️⃣ Checking open positions"
echo "---------------------------------"

curl -s $BASE_URL/positions
echo -e "\n"

echo "---------------------------------"
echo "3️⃣ Waiting for market movement..."
echo "---------------------------------"

sleep 15

echo "---------------------------------"
echo "4️⃣ Checking positions after market move"
echo "---------------------------------"

curl -s $BASE_URL/positions
echo -e "\n"

echo "---------------------------------"
echo "5️⃣ Closing position"
echo "---------------------------------"

curl -s -X POST $BASE_URL/position/close \
-H "Content-Type: application/json" \
-d "{\"position_id\":\"$POSITION_ID\"}"

echo -e "\n"

echo "---------------------------------"
echo "6️⃣ Final positions state"
echo "---------------------------------"

curl -s $BASE_URL/positions
echo -e "\n"

echo "---------------------------------"
echo "7️⃣ Checking price feed"
echo "---------------------------------"

curl -s $BASE_URL/price
echo -e "\n"

echo "---------------------------------"
echo " 8️⃣ Checking balance"
echo "---------------------------------"

curl -s $BASE_URL/balance
echo -e "\n"

echo "---------------------------------"
echo " 9️⃣ Checking Trade history"
echo "---------------------------------"

curl -s $BASE_URL/trade-history
echo -e "\n"

echo "---------------------------------"
echo " 🔟 Checking Funding Rate"
echo "---------------------------------"
curl -s $BASE_URL/funding-rate
echo -e "\n"

echo "---------------------------------"
echo "✅ Test Complete"
echo "---------------------------------"