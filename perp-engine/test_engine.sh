#!/bin/bash

BASE_URL="http://127.0.0.1:8080"

echo ""
echo "Testing Perp Engine API"
echo ""

echo "---------------------------------"
echo "0️⃣ Login - Getting JWT Token"
echo "---------------------------------"

LOGIN_RESPONSE=$(curl -s -X POST $BASE_URL/auth/login \
-H "Content-Type: application/json" \
-d '{"username":"admin","password":"secret"}')

echo $LOGIN_RESPONSE
echo -e "\n"

TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

echo "Extracted Token:"
echo $TOKEN
echo -e "\n"

if [ -z "$TOKEN" ]; then
    echo "❌ Failed to get token. Exiting."
    exit 1
fi

echo "⏳ Waiting for price feed to initialize..."
for i in {1..10}; do
    PRICE=$(curl -s $BASE_URL/price -H "Authorization: Bearer $TOKEN" | grep -o '"current_price":"[^"]*"' | cut -d'"' -f4)
    if [ "$PRICE" != "0.00000000" ] && [ -n "$PRICE" ]; then
        echo "✅ Price ready: $PRICE"
        break
    fi
    echo "Attempt $i: price not ready, waiting..."
    sleep 2
done

echo "---------------------------------"
echo "1️⃣ Opening position"
echo "---------------------------------"

OPEN_RESPONSE=$(curl -s -X POST $BASE_URL/position/open \
-H "Content-Type: application/json" \
-H "Authorization: Bearer $TOKEN" \
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

curl -s $BASE_URL/positions \
-H "Authorization: Bearer $TOKEN"
echo -e "\n"

echo "---------------------------------"
echo "3️⃣ Waiting for market movement..."
echo "---------------------------------"

sleep 15

echo "---------------------------------"
echo "4️⃣ Checking positions after market move"
echo "---------------------------------"

curl -s $BASE_URL/positions \
-H "Authorization: Bearer $TOKEN"
echo -e "\n"

echo "---------------------------------"
echo "5️⃣ Closing position"
echo "---------------------------------"

curl -s -X POST $BASE_URL/position/close \
-H "Content-Type: application/json" \
-H "Authorization: Bearer $TOKEN" \
-d "{\"position_id\":\"$POSITION_ID\"}"

echo -e "\n"

echo "---------------------------------"
echo "6️⃣ Final positions state"
echo "---------------------------------"

curl -s $BASE_URL/positions \
-H "Authorization: Bearer $TOKEN"
echo -e "\n"

echo "---------------------------------"
echo "7️⃣ Checking price feed"
echo "---------------------------------"

curl -s $BASE_URL/price \
-H "Authorization: Bearer $TOKEN"
echo -e "\n"

echo "---------------------------------"
echo "8️⃣ Checking balance"
echo "---------------------------------"

curl -s $BASE_URL/balance \
-H "Authorization: Bearer $TOKEN"
echo -e "\n"

echo "---------------------------------"
echo "9️⃣ Checking Trade history"
echo "---------------------------------"

curl -s $BASE_URL/trade-history \
-H "Authorization: Bearer $TOKEN"
echo -e "\n"

echo "---------------------------------"
echo "🔟 Checking Funding Rate"
echo "---------------------------------"

curl -s $BASE_URL/funding-rate \
-H "Authorization: Bearer $TOKEN"
echo -e "\n"

echo "---------------------------------"
echo "✅ Test Complete"
echo "---------------------------------"