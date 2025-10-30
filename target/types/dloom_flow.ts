/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/dloom_flow.json`.
 */
export type DloomFlow = {
  "address": "6fG9BGsHZjsV9Rie5fm2r9J9cfsqBG8kgTAicbHQtCwH",
  "metadata": {
    "name": "dloomFlow",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "addLiquidity",
      "discriminator": [
        181,
        157,
        89,
        67,
        143,
        182,
        52,
        72
      ],
      "accounts": [
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "position",
          "writable": true
        },
        {
          "name": "owner",
          "writable": true,
          "signer": true,
          "relations": [
            "position",
            "userTokenAAccount",
            "userTokenBAccount"
          ]
        },
        {
          "name": "tokenAMint"
        },
        {
          "name": "tokenBMint"
        },
        {
          "name": "userTokenAAccount",
          "writable": true
        },
        {
          "name": "userTokenBAccount",
          "writable": true
        },
        {
          "name": "tokenAVault",
          "writable": true
        },
        {
          "name": "tokenBVault",
          "writable": true
        },
        {
          "name": "tokenAProgram"
        },
        {
          "name": "tokenBProgram"
        }
      ],
      "args": [
        {
          "name": "startBinId",
          "type": "i32"
        },
        {
          "name": "liquidityPerBin",
          "type": "u128"
        }
      ]
    },
    {
      "name": "burnEmptyPosition",
      "discriminator": [
        168,
        131,
        222,
        80,
        72,
        245,
        157,
        79
      ],
      "accounts": [
        {
          "name": "owner",
          "writable": true,
          "signer": true,
          "relations": [
            "position"
          ]
        },
        {
          "name": "position",
          "writable": true
        },
        {
          "name": "positionMint",
          "writable": true
        },
        {
          "name": "userPositionNftAccount",
          "writable": true
        },
        {
          "name": "tokenProgram"
        }
      ],
      "args": []
    },
    {
      "name": "getPrice",
      "discriminator": [
        238,
        38,
        193,
        106,
        228,
        32,
        210,
        33
      ],
      "accounts": [
        {
          "name": "pool"
        }
      ],
      "args": [
        {
          "name": "binId",
          "type": "i32"
        }
      ],
      "returns": "u128"
    },
    {
      "name": "initializeBin",
      "discriminator": [
        193,
        128,
        145,
        146,
        182,
        247,
        87,
        8
      ],
      "accounts": [
        {
          "name": "bin",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  98,
                  105,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "arg",
                "path": "binId"
              }
            ]
          }
        },
        {
          "name": "pool"
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "binId",
          "type": "i32"
        }
      ]
    },
    {
      "name": "initializePool",
      "discriminator": [
        95,
        180,
        10,
        172,
        84,
        174,
        232,
        40
      ],
      "accounts": [
        {
          "name": "tokenAMint"
        },
        {
          "name": "tokenBMint"
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "pool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "tokenAMint"
              },
              {
                "kind": "account",
                "path": "tokenBMint"
              },
              {
                "kind": "arg",
                "path": "binStep"
              }
            ]
          }
        },
        {
          "name": "tokenAVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "account",
                "path": "tokenAMint"
              }
            ]
          }
        },
        {
          "name": "tokenBVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "account",
                "path": "tokenBMint"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "rent",
          "address": "SysvarRent111111111111111111111111111111111"
        },
        {
          "name": "tokenAProgram"
        },
        {
          "name": "tokenBProgram"
        }
      ],
      "args": [
        {
          "name": "binStep",
          "type": "u16"
        },
        {
          "name": "feeRate",
          "type": "u16"
        },
        {
          "name": "initialBinId",
          "type": "i32"
        }
      ]
    },
    {
      "name": "modifyLiquidity",
      "discriminator": [
        14,
        214,
        168,
        41,
        55,
        15,
        53,
        17
      ],
      "accounts": [
        {
          "name": "owner",
          "writable": true,
          "signer": true,
          "relations": [
            "oldPosition",
            "newPosition"
          ]
        },
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "oldPosition",
          "writable": true
        },
        {
          "name": "newPosition",
          "writable": true
        },
        {
          "name": "tokenAMint"
        },
        {
          "name": "tokenBMint"
        },
        {
          "name": "userTokenAAccount",
          "writable": true
        },
        {
          "name": "userTokenBAccount",
          "writable": true
        },
        {
          "name": "tokenAVault",
          "writable": true
        },
        {
          "name": "tokenBVault",
          "writable": true
        },
        {
          "name": "tokenAProgram"
        },
        {
          "name": "tokenBProgram"
        }
      ],
      "args": [
        {
          "name": "minSurplusAOut",
          "type": "u64"
        },
        {
          "name": "minSurplusBOut",
          "type": "u64"
        }
      ]
    },
    {
      "name": "openPosition",
      "discriminator": [
        135,
        128,
        47,
        77,
        15,
        152,
        240,
        49
      ],
      "accounts": [
        {
          "name": "pool"
        },
        {
          "name": "position",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "positionMint"
              }
            ]
          }
        },
        {
          "name": "owner",
          "writable": true,
          "signer": true
        },
        {
          "name": "positionMint",
          "writable": true,
          "signer": true
        },
        {
          "name": "userPositionNftAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "owner"
              },
              {
                "kind": "const",
                "value": [
                  6,
                  221,
                  246,
                  225,
                  215,
                  101,
                  161,
                  147,
                  217,
                  203,
                  225,
                  70,
                  206,
                  235,
                  121,
                  172,
                  28,
                  180,
                  133,
                  237,
                  95,
                  91,
                  55,
                  145,
                  58,
                  140,
                  245,
                  133,
                  126,
                  255,
                  0,
                  169
                ]
              },
              {
                "kind": "account",
                "path": "positionMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "tokenAMint"
        },
        {
          "name": "tokenBMint"
        },
        {
          "name": "metadataAccount",
          "writable": true
        },
        {
          "name": "masterEditionAccount",
          "writable": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "tokenMetadataProgram",
          "address": "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
        },
        {
          "name": "rent",
          "address": "SysvarRent111111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "lowerBinId",
          "type": "i32"
        },
        {
          "name": "upperBinId",
          "type": "i32"
        }
      ]
    },
    {
      "name": "removeLiquidity",
      "discriminator": [
        80,
        85,
        209,
        72,
        24,
        206,
        177,
        108
      ],
      "accounts": [
        {
          "name": "owner",
          "writable": true,
          "signer": true,
          "relations": [
            "position"
          ]
        },
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "position",
          "writable": true
        },
        {
          "name": "tokenAMint"
        },
        {
          "name": "tokenBMint"
        },
        {
          "name": "userTokenAAccount",
          "writable": true
        },
        {
          "name": "userTokenBAccount",
          "writable": true
        },
        {
          "name": "tokenAVault",
          "writable": true
        },
        {
          "name": "tokenBVault",
          "writable": true
        },
        {
          "name": "tokenAProgram"
        },
        {
          "name": "tokenBProgram"
        }
      ],
      "args": [
        {
          "name": "liquidityToRemove",
          "type": "u128"
        },
        {
          "name": "minAmountA",
          "type": "u64"
        },
        {
          "name": "minAmountB",
          "type": "u64"
        }
      ]
    },
    {
      "name": "swap",
      "discriminator": [
        248,
        198,
        158,
        145,
        225,
        117,
        135,
        200
      ],
      "accounts": [
        {
          "name": "trader",
          "writable": true,
          "signer": true
        },
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "tokenAMint"
        },
        {
          "name": "tokenBMint"
        },
        {
          "name": "userSourceTokenAccount",
          "writable": true
        },
        {
          "name": "userDestinationTokenAccount",
          "writable": true
        },
        {
          "name": "sourceVault",
          "writable": true
        },
        {
          "name": "destinationVault",
          "writable": true
        },
        {
          "name": "tokenAProgram"
        },
        {
          "name": "tokenBProgram"
        }
      ],
      "args": [
        {
          "name": "amountIn",
          "type": "u64"
        },
        {
          "name": "minAmountOut",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "bin",
      "discriminator": [
        254,
        7,
        131,
        225,
        223,
        87,
        157,
        218
      ]
    },
    {
      "name": "pool",
      "discriminator": [
        241,
        154,
        109,
        4,
        17,
        177,
        109,
        188
      ]
    },
    {
      "name": "position",
      "discriminator": [
        170,
        188,
        143,
        228,
        122,
        64,
        247,
        208
      ]
    }
  ],
  "events": [
    {
      "name": "liquidityRebalanced",
      "discriminator": [
        86,
        57,
        142,
        225,
        181,
        136,
        251,
        109
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "invalidParameters",
      "msg": "The provided fee and bin_step parameters are not on the whitelist."
    },
    {
      "code": 6001,
      "name": "invalidMintOrder",
      "msg": "The mint addresses are not in the correct canonical order. Token A must be less than Token B."
    },
    {
      "code": 6002,
      "name": "invalidMint",
      "msg": "The provided mint does not match the pool's mint."
    },
    {
      "code": 6003,
      "name": "invalidBinRange",
      "msg": "The lower bin ID must be less than the upper bin ID."
    },
    {
      "code": 6004,
      "name": "zeroLiquidity",
      "msg": "Liquidity to deposit must be greater than zero."
    },
    {
      "code": 6005,
      "name": "slippageExceeded",
      "msg": "The market price moved unfavorably, exceeding your slippage tolerance."
    },
    {
      "code": 6006,
      "name": "unauthorized",
      "msg": "The signer is not the authorized owner of this position."
    },
    {
      "code": 6007,
      "name": "insufficientLiquidity",
      "msg": "The amount of liquidity to remove exceeds the amount in the position."
    },
    {
      "code": 6008,
      "name": "positionNotEmpty",
      "msg": "Cannot operate on a position that has no liquidity."
    },
    {
      "code": 6009,
      "name": "zeroAmount",
      "msg": "Input amount for a swap must be greater than zero."
    },
    {
      "code": 6010,
      "name": "invalidVault",
      "msg": "The provided vault account does not match the pool's vault."
    },
    {
      "code": 6011,
      "name": "invalidBinId",
      "msg": "The provided bin IDs must be a multiple of the pool's bin_step."
    },
    {
      "code": 6012,
      "name": "rangeTooWide",
      "msg": "The specified bin range is wider than the allowed maximum."
    },
    {
      "code": 6013,
      "name": "mathOverflow",
      "msg": "Math operation overflowed or underflowed."
    },
    {
      "code": 6014,
      "name": "invalidBinStep",
      "msg": "The provided bin step value is invalid (e.g., zero)."
    },
    {
      "code": 6015,
      "name": "insufficientLiquidityForSwap",
      "msg": "Not enough liquidity in the pool to complete the swap."
    },
    {
      "code": 6016,
      "name": "invalidBinCount",
      "msg": "The number of bins provided does not match the position's range."
    },
    {
      "code": 6017,
      "name": "invalidBinAccount",
      "msg": "A provided bin account does not have the expected address."
    },
    {
      "code": 6018,
      "name": "invalidPool",
      "msg": "The provided position account does not belong to the specified pool."
    }
  ],
  "types": [
    {
      "name": "bin",
      "serialization": "bytemuck",
      "repr": {
        "kind": "c"
      },
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "liquidity",
            "type": "u128"
          },
          {
            "name": "feeGrowthPerUnitA",
            "type": "u128"
          },
          {
            "name": "feeGrowthPerUnitB",
            "type": "u128"
          }
        ]
      }
    },
    {
      "name": "liquidityRebalanced",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "oldPosition",
            "type": "pubkey"
          },
          {
            "name": "newPosition",
            "type": "pubkey"
          },
          {
            "name": "liquidityMoved",
            "type": "u128"
          },
          {
            "name": "newLowerBinId",
            "type": "i32"
          },
          {
            "name": "newUpperBinId",
            "type": "i32"
          }
        ]
      }
    },
    {
      "name": "pool",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "tokenAMint",
            "type": "pubkey"
          },
          {
            "name": "tokenBMint",
            "type": "pubkey"
          },
          {
            "name": "tokenAVault",
            "type": "pubkey"
          },
          {
            "name": "tokenBVault",
            "type": "pubkey"
          },
          {
            "name": "activeBinId",
            "type": "i32"
          },
          {
            "name": "binStep",
            "type": "u16"
          },
          {
            "name": "feeRate",
            "type": "u16"
          },
          {
            "name": "reservesA",
            "type": "u64"
          },
          {
            "name": "reservesB",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "position",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "lowerBinId",
            "type": "i32"
          },
          {
            "name": "upperBinId",
            "type": "i32"
          },
          {
            "name": "liquidity",
            "type": "u128"
          },
          {
            "name": "positionMint",
            "type": "pubkey"
          },
          {
            "name": "feeGrowthSnapshotA",
            "type": "u128"
          },
          {
            "name": "feeGrowthSnapshotB",
            "type": "u128"
          }
        ]
      }
    }
  ]
};
