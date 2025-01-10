#!/bin/bash

curl 'https://api.paraswap.io/prices/?version=6.2&srcToken=0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48&destToken=0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE&amount=100000000000000000000&srcDecimals=6&destDecimals=18&side=BUY&network=1&otherExchangePrices=true&partner=paraswap.io&userAddress=0x0000000000000000000000000000000000000000' \
  -H 'accept: application/json, text/plain, */*'
