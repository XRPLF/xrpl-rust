const TEST_IOU_CASES = [
    [
        {
            "value": "0",
            "currency": "USD",
            "issuer": "rDgZZ3wyprx4ZqrGQUkquE9Fs2Xs8XBcdw",
        },
        "80000000000000000000000000000000000000005553440000"
        "0000008B1CE810C13D6F337DAC85863B3D70265A24DF44",
    ],
    [
        {
            "value": "1",
            "currency": "USD",
            "issuer": "rDgZZ3wyprx4ZqrGQUkquE9Fs2Xs8XBcdw",
        },
        "D4838D7EA4C680000000000000000000000000005553440000"
        "0000008B1CE810C13D6F337DAC85863B3D70265A24DF44",
    ],
    [
        {
            "value": "2",
            "currency": "USD",
            "issuer": "rDgZZ3wyprx4ZqrGQUkquE9Fs2Xs8XBcdw",
        },
        "D4871AFD498D00000000000000000000000000005553440000"
        "0000008B1CE810C13D6F337DAC85863B3D70265A24DF44",
    ],
    [
        {
            "value": "-2",
            "currency": "USD",
            "issuer": "rDgZZ3wyprx4ZqrGQUkquE9Fs2Xs8XBcdw",
        },
        "94871AFD498D00000000000000000000000000005553440000"
        "0000008B1CE810C13D6F337DAC85863B3D70265A24DF44",
    ],
    [
        {
            "value": "2.1",
            "currency": "USD",
            "issuer": "rDgZZ3wyprx4ZqrGQUkquE9Fs2Xs8XBcdw",
        },
        "D48775F05A0740000000000000000000000000005553440000"
        "0000008B1CE810C13D6F337DAC85863B3D70265A24DF44",
    ],
    [
        {
            "currency": "XRP",
            "value": "2.1",
            "issuer": "rrrrrrrrrrrrrrrrrrrrrhoLvTp",
        },
        "D48775F05A07400000000000000000000000000000000000"
        "000000000000000000000000000000000000000000000000",
    ],
    [
        {
            "currency": "USD",
            "value": "1111111111111111",
            "issuer": "rrrrrrrrrrrrrrrrrrrrBZbvji",
        },
        "D843F28CB71571C700000000000000000000000055534400"
        "000000000000000000000000000000000000000000000001",
    ],
]

const TEST_XRP_CASES = [
    ["100", "4000000000000064"],
    ["100000000000000000", "416345785D8A0000"],
];
