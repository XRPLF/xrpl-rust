#ifndef SECP256K1_MPT_H
#define SECP256K1_MPT_H

#include "mpt_protocol.h"
#include <secp256k1.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Generates a new secp256k1 key pair.
 */
SECP256K1_API int
secp256k1_elgamal_generate_keypair(
    secp256k1_context const* ctx,
    unsigned char* privkey,
    secp256k1_pubkey* pubkey);

/**
 * @brief Encrypts a 64-bit amount using ElGamal.
 */
SECP256K1_API int
secp256k1_elgamal_encrypt(
    secp256k1_context const* ctx,
    secp256k1_pubkey* c1,
    secp256k1_pubkey* c2,
    secp256k1_pubkey const* pubkey_Q,
    uint64_t amount,
    unsigned char const* blinding_factor);

/**
 * @brief Decrypts an ElGamal ciphertext to recover the amount.
 *
 * The function uses a linear discrete-logarithm search over the caller-
 * specified range [range_low, range_high] and can only successfully recover
 * plaintext amounts within that range. If the ciphertext encrypts a value
 * outside the range, the search exhausts its iterations and the function
 * returns 0 (not found). The recommended default range is [0, 1,000,000].
 *
 * To mitigate amount-dependent timing side channels (TOB-RIPCTX-1 / -4,
 * reintroduced as TOB-RIPCTXR-1), the search executes with a fixed
 * iteration count: it always runs to completion over the full specified
 * range regardless of where (or whether) the match is found. This is a
 * fixed iteration count, not strict constant time: the underlying
 * libsecp256k1 curve operations are inherently variable-time, so this
 * function does not protect against attackers that can observe
 * microarchitectural variation of individual point operations. In shared /
 * multitenant deployments where such attacks are relevant, callers should
 * not expose decryption latency to untrusted observers.
 *
 * Architectural note: on-chain validators and verifiers never decrypt
 * ciphertexts; this function is intended for testing and basic client-
 * side operations. Off-chain applications (wallets, audit tooling) that
 * need to decrypt larger balances should use more efficient discrete
 * logarithm algorithms such as baby-step giant-step (O(sqrt(n))) or
 * Pollard's kangaroo.
 *
 * @param[in]  ctx        A pointer to a valid secp256k1 context.
 * @param[out] amount     Set to the decrypted plaintext amount on success.
 * @param[in]  c1         The C1 component of the ElGamal ciphertext.
 * @param[in]  c2         The C2 component of the ElGamal ciphertext.
 * @param[in]  privkey    The 32-byte ElGamal private key.
 * @param[in]  range_low  Lower bound of the search range (inclusive).
 * @param[in]  range_high Upper bound of the search range (inclusive).
 *                        Must be >= range_low and must not be UINT64_MAX;
 *                        either condition returns 0 immediately.  UINT64_MAX
 *                        is rejected because the loop runs
 *                        range_high - max(1, range_low) + 1 iterations — a
 *                        UINT64_MAX upper bound would require up to 2^64 - 1
 *                        iterations (effectively infinite).  Use
 *                        secp256k1_elgamal_decrypt_bsgs for larger ranges.
 *                        Recommended default: range_low=0, range_high=1000000.
 *
 * @return 1 if the ciphertext decrypts to an amount in [range_low, range_high]
 *         and `*amount` is set; 0 otherwise. A 0 return covers both
 *         "amount out of range" and "internal failure".
 */
SECP256K1_API int
secp256k1_elgamal_decrypt(
    secp256k1_context const* ctx,
    uint64_t* amount,
    secp256k1_pubkey const* c1,
    secp256k1_pubkey const* c2,
    unsigned char const* privkey,
    uint64_t range_low,
    uint64_t range_high);

/**
 * @brief Homomorphically adds two ElGamal ciphertexts.
 */
SECP256K1_API int
secp256k1_elgamal_add(
    secp256k1_context const* ctx,
    secp256k1_pubkey* sum_c1,
    secp256k1_pubkey* sum_c2,
    secp256k1_pubkey const* a_c1,
    secp256k1_pubkey const* a_c2,
    secp256k1_pubkey const* b_c1,
    secp256k1_pubkey const* b_c2);

/**
 * @brief Homomorphically subtracts two ElGamal ciphertexts.
 */
SECP256K1_API int
secp256k1_elgamal_subtract(
    secp256k1_context const* ctx,
    secp256k1_pubkey* diff_c1,
    secp256k1_pubkey* diff_c2,
    secp256k1_pubkey const* a_c1,
    secp256k1_pubkey const* a_c2,
    secp256k1_pubkey const* b_c1,
    secp256k1_pubkey const* b_c2);

/**
 * @brief Generates the canonical encrypted zero for a given MPT token instance.
 *
 * This ciphertext represents a zero balance for a specific account's holding
 * of a token defined by its MPTokenIssuanceID.
 *
 * @param[in]   ctx             A pointer to a valid secp256k1 context.
 * @param[out]  enc_zero_c1     The C1 component of the canonical ciphertext.
 * @param[out]  enc_zero_c2     The C2 component of the canonical ciphertext.
 * @param[in]   pubkey          The ElGamal public key of the account holder.
 * @param[in]   account_id      A pointer to the 20-byte AccountID.
 * @param[in]   mpt_issuance_id A pointer to the 24-byte MPTokenIssuanceID.
 *
 * @return 1 on success, 0 on failure.
 */
SECP256K1_API int
generate_canonical_encrypted_zero(
    secp256k1_context const* ctx,
    secp256k1_pubkey* enc_zero_c1,
    secp256k1_pubkey* enc_zero_c2,
    secp256k1_pubkey const* pubkey,
    unsigned char const* account_id,      // 20 bytes
    unsigned char const* mpt_issuance_id  // 24 bytes
);

/**
 * @brief Computes a Pedersen Commitment: C = value*G + blinding_factor*H.
 *
 * This function creates the commitment point (C) that the Bulletproof proves
 * the range of.
 *
 * @param[in]   ctx             A pointer to the context.
 * @param[out]  commitment_C    The resulting commitment point C.
 * @param[in]   value           The secret amount v (uint64_t).
 * @param[in]   blinding_factor The secret randomness r (32 bytes).
 * @param[in]   h_generator     The Pedersen blinding generator H, as returned
 *                              by secp256k1_mpt_get_h_generator(). This MUST
 *                              be the standardized nothing-up-my-sleeve H
 *                              generator; it must NOT be a holder, issuer,
 *                              auditor, or recipient encryption public key.
 *                              Pedersen binding requires that the discrete
 *                              log of H be unknown to all parties; supplying
 *                              a key whose discrete log is known to any party
 *                              breaks binding and lets that party compute
 *                              alternate openings of the same commitment.
 *
 * @return 1 on success, 0 on failure.
 */
SECP256K1_API int
secp256k1_bulletproof_create_commitment(
    secp256k1_context const* ctx,
    secp256k1_pubkey* commitment_C,
    uint64_t value,
    unsigned char const* blinding_factor,
    secp256k1_pubkey const* h_generator);

int
secp256k1_bulletproof_prove(
    secp256k1_context const* ctx,
    unsigned char* proof_out,
    size_t* proof_len,
    uint64_t value,
    unsigned char const* blinding_factor,
    secp256k1_pubkey const* h_generator,
    unsigned char const* context_id, /* <--- AND HERE */
    unsigned int proof_type);

int
secp256k1_bulletproof_verify(
    secp256k1_context const* ctx,
    secp256k1_pubkey const* G_vec,
    secp256k1_pubkey const* H_vec,
    unsigned char const* proof,
    size_t proof_len,
    secp256k1_pubkey const* commitment_C,
    secp256k1_pubkey const* h_generator, /* This is generator H */
    unsigned char const* context_id);
/**
 * Verifies that (c1, c2) is a valid ElGamal encryption of 'amount'
 * for 'pubkey_Q' using the revealed 'blinding_factor'.
 */
int
secp256k1_elgamal_verify_encryption(
    secp256k1_context const* ctx,
    secp256k1_pubkey const* c1,
    secp256k1_pubkey const* c2,
    secp256k1_pubkey const* pubkey_Q,
    uint64_t amount,
    unsigned char const* blinding_factor);

/** Proof of Knowledge of Secret Key for Registration.
 *  Compact form: (e, s) in Z_q^2 = 64 bytes.
 *  Domain: "CMPT_POK_SK_REGISTER"
 *  Note: takes (pk, sk), the reverse of the secp256k1_rotate_*_prove family,
 *  which takes (sk, pk).  Kept as-is for ABI compatibility. */
#define SECP256K1_POK_SK_PROOF_SIZE 64

SECP256K1_API int
secp256k1_mpt_pok_sk_prove(
    secp256k1_context const* ctx,
    unsigned char* proof, /* Expected size: 64 bytes */
    secp256k1_pubkey const* pk,
    unsigned char const* sk,
    unsigned char const* context_id);

SECP256K1_API int
secp256k1_mpt_pok_sk_verify(
    secp256k1_context const* ctx,
    unsigned char const* proof, /* Expected size: 64 bytes */
    secp256k1_pubkey const* pk,
    unsigned char const* context_id);

/**
 * Compute a Pedersen Commitment: PC = m*G + rho*H
 * Returns 1 on success, 0 on failure.
 */
int
secp256k1_mpt_pedersen_commit(
    secp256k1_context const* ctx,
    secp256k1_pubkey* commitment,
    uint64_t amount,
    unsigned char const* blinding_factor_rho /* 32 bytes */
);

/** Get the standardized H generator for Pedersen Commitments */
int
secp256k1_mpt_get_h_generator(secp256k1_context const* ctx, secp256k1_pubkey* h);

/**
 * @brief Generates a vector of N independent NUMS generators.
 */
int
secp256k1_mpt_get_generator_vector(
    secp256k1_context const* ctx,
    secp256k1_pubkey* vec,
    size_t n,
    unsigned char const* label,
    size_t label_len);

void
secp256k1_mpt_scalar_add(unsigned char* res, unsigned char const* a, unsigned char const* b);
void
secp256k1_mpt_scalar_mul(unsigned char* res, unsigned char const* a, unsigned char const* b);
/* Computes the modular inverse of a scalar. Returns 1 on success,
 * 0 if the input is zero (inverse undefined). Callers must check
 * the return value. */

int
secp256k1_mpt_scalar_inverse(unsigned char* res, unsigned char const* in);
void
secp256k1_mpt_scalar_negate(unsigned char* res, unsigned char const* in);
void
secp256k1_mpt_scalar_reduce32(unsigned char out32[32], unsigned char const in32[32]);

int
secp256k1_bulletproof_prove_agg(
    secp256k1_context const* ctx,
    unsigned char* proof_out,
    size_t* proof_len,
    uint64_t const* values,
    unsigned char const* blindings_flat,
    size_t m,
    secp256k1_pubkey const* h_generator,
    unsigned char const* context_id);
int
secp256k1_bulletproof_verify_agg(
    secp256k1_context const* ctx,
    secp256k1_pubkey const* G_vec, /* length n = 64*m */
    secp256k1_pubkey const* H_vec, /* length n = 64*m */
    unsigned char const* proof,
    size_t proof_len,
    secp256k1_pubkey const* commitment_C_vec, /* length m */
    size_t m,
    secp256k1_pubkey const* h_generator,
    unsigned char const* context_id);

/*
================================================================================
|                                                                              |
|               AND-COMPOSED COMPACT SIGMA PROOF (STANDARD EG)                |
|                                                                              |
================================================================================
 *
 * Combines ciphertext equality, Pedersen linkage, and balance verification
 * into a single sigma protocol under a shared Fiat-Shamir challenge.
 *
 * Language: exists (r, m, sk_A, rho, b) in Z_q^5 such that:
 *   C1          = r*G
 *   C_{2,i}     = m*G + r*pk_i   for i = 1..n
 *   PC_m        = m*G + r*H
 *   pk_A        = sk_A*G
 *   PC_b        = b*G + rho*H
 *   B2 - b*G    = sk_A*B1
 *
 * Compact proof: (e, z_m, z_r, z_b, z_rho, z_sk) in Z_q^6 = 192 bytes.
 * Fiat-Shamir domain: "CMPT_SEND_SIGMA"
 */

/** Serialized size of the compact standard proof in bytes. */
#define SECP256K1_COMPACT_STANDARD_PROOF_SIZE 192

/**
 * @brief Generate a compact AND-composed sigma proof for standard EC-ElGamal.
 *
 * proof_out must point to a buffer of SECP256K1_COMPACT_STANDARD_PROOF_SIZE
 * bytes. context_id is an optional 32-byte transaction context (may be NULL).
 */
SECP256K1_API int
secp256k1_compact_standard_prove(
    secp256k1_context const* ctx,
    unsigned char* proof_out,
    uint64_t amount,
    uint64_t balance,
    unsigned char const* r_shared,
    unsigned char const* sk_A,
    unsigned char const* r_b,
    size_t n,
    secp256k1_pubkey const* C1,
    secp256k1_pubkey const* C2_vec,
    secp256k1_pubkey const* Pk_vec,
    secp256k1_pubkey const* PC_m,
    secp256k1_pubkey const* pk_A,
    secp256k1_pubkey const* PC_b,
    secp256k1_pubkey const* B1,
    secp256k1_pubkey const* B2,
    unsigned char const* context_id);

/**
 * @brief Verify a compact AND-composed sigma proof for standard EC-ElGamal.
 *
 * Returns 1 if the proof is valid, 0 otherwise.
 */
SECP256K1_API int
secp256k1_compact_standard_verify(
    secp256k1_context const* ctx,
    unsigned char const* proof,
    size_t n,
    secp256k1_pubkey const* C1,
    secp256k1_pubkey const* C2_vec,
    secp256k1_pubkey const* Pk_vec,
    secp256k1_pubkey const* PC_m,
    secp256k1_pubkey const* pk_A,
    secp256k1_pubkey const* PC_b,
    secp256k1_pubkey const* B1,
    secp256k1_pubkey const* B2,
    unsigned char const* context_id);

/*
================================================================================
|                                                                              |
|            COMPACT SIGMA PROOF — CLAWBACK                                   |
|                                                                              |
================================================================================
 *
 * Proves the issuer knows sk_iss consistent with the on-ledger mirror
 * ciphertext (C1, C2) and the publicly declared amount m:
 *   P_iss      = sk_iss * G
 *   C2 - m*G   = sk_iss * C1
 *
 * Compact proof: (e, z_sk) in Z_q^2 = 64 bytes.
 * Fiat-Shamir domain: "CMPT_CLAWBACK_SIGMA"
 */

#define SECP256K1_COMPACT_CLAWBACK_PROOF_SIZE 64

SECP256K1_API int
secp256k1_compact_clawback_prove(
    secp256k1_context const* ctx,
    unsigned char* proof_out,
    uint64_t amount,
    unsigned char const* sk_iss,
    secp256k1_pubkey const* P_iss,
    secp256k1_pubkey const* C1,
    secp256k1_pubkey const* C2,
    unsigned char const* context_id);

SECP256K1_API int
secp256k1_compact_clawback_verify(
    secp256k1_context const* ctx,
    unsigned char const* proof,
    uint64_t amount,
    secp256k1_pubkey const* P_iss,
    secp256k1_pubkey const* C1,
    secp256k1_pubkey const* C2,
    unsigned char const* context_id);

/*
================================================================================
|                                                                              |
|            COMPACT SIGMA PROOF — CONVERTBACK                                |
|                                                                              |
================================================================================
 *
 * AND-composed proof for balance linkage in a ConvertBack withdrawal.
 * The withdrawal ciphertext (C1_w, C2_w) is verified deterministically
 * using the publicly disclosed r_w (BlindingFactor field), so the sigma
 * proof covers only key ownership, balance decryption, and commitment.
 *
 * Language: exists (b, sk_A, rho) in Z_q^3 such that:
 *   P_A      = sk_A*G
 *   B2 - b*G = sk_A*B1
 *   PC_b     = b*G + rho*H
 *
 * Compact proof: (e, z_b, z_rho, z_sk) in Z_q^4 = 128 bytes.
 * Fiat-Shamir domain: "CMPT_CONVERTBACK_SIGMA"
 *
 * The caller must separately verify the withdrawal ciphertext:
 *   C1_w == r_w*G  and  C2_w == m*G + r_w*P_A
 * using secp256k1_elgamal_verify_encryption() or equivalent.
 */

#define SECP256K1_COMPACT_CONVERTBACK_PROOF_SIZE 128

SECP256K1_API int
secp256k1_compact_convertback_prove(
    secp256k1_context const* ctx,
    unsigned char* proof_out,
    uint64_t balance,
    unsigned char const* sk_A,
    unsigned char const* rho,
    secp256k1_pubkey const* pk_A,
    secp256k1_pubkey const* B1,
    secp256k1_pubkey const* B2,
    secp256k1_pubkey const* PC_b,
    unsigned char const* context_id);

SECP256K1_API int
secp256k1_compact_convertback_verify(
    secp256k1_context const* ctx,
    unsigned char const* proof,
    secp256k1_pubkey const* pk_A,
    secp256k1_pubkey const* B1,
    secp256k1_pubkey const* B2,
    secp256k1_pubkey const* PC_b,
    unsigned char const* context_id);

/* ---------------------------------------------------------------------------
 * ElGamal key rotation proofs
 *
 * Five of the key rotation variants share one relation, differing only in which
 * ciphertext plays the decryption side, which key plays the re-encryption
 * target, and the domain separation tag:
 *
 *   exists (b, sk, r) in Z_q^3 such that:
 *     pk_old      = sk*G
 *     C2 - sk*C1  = b*G
 *     D1          = r*G
 *     D2          = b*G + r*pk_target
 *
 * They therefore share an internal implementation and are all 4 scalars.
 * Every element of every statement below is expected to be read from
 * authenticated ledger state, never taken from a transaction field.
 * ------------------------------------------------------------------------- */

/** Compact proof size for the shared re-encryption relation: 4 scalars. */
#define SECP256K1_ROTATE_REENCRYPT_PROOF_SIZE 128

/**
 * pi_mi - issuer-anchored migration of the issuer mirror.
 * Domain tag "CMPT_KEY_ROTATION_MIRROR_ISSUER_ONLY".
 * Decryption side is the current issuer mirror (E1, E2) under the current
 * pk_I; the new issuer mirror (E1', E2') is created under pk_I'. Both pk_I
 * and (E1, E2) are resolved entirely from ledger state (XLS-99's fix for
 * Finding f:pki): IssuerMirrorEncryptionKey on the holder's MPToken,
 * falling back to InitialIssuerEncryptionKey on the MPTokenIssuance. No
 * submitter-chosen previous-key transaction field is involved.
 */
SECP256K1_API int
secp256k1_rotate_mirror_issuer_prove(
    secp256k1_context const* ctx,
    unsigned char* proof_out,
    uint64_t balance,
    unsigned char const* sk_I,
    unsigned char const* r_new,
    secp256k1_pubkey const* pk_I,
    secp256k1_pubkey const* E1,
    secp256k1_pubkey const* E2,
    secp256k1_pubkey const* pk_I_new,
    secp256k1_pubkey const* E1_new,
    secp256k1_pubkey const* E2_new,
    unsigned char const* context_id);

SECP256K1_API int
secp256k1_rotate_mirror_issuer_verify(
    secp256k1_context const* ctx,
    unsigned char const* proof,
    secp256k1_pubkey const* pk_I,
    secp256k1_pubkey const* E1,
    secp256k1_pubkey const* E2,
    secp256k1_pubkey const* pk_I_new,
    secp256k1_pubkey const* E1_new,
    secp256k1_pubkey const* E2_new,
    unsigned char const* context_id);

/**
 * pi_mh - holder self-migration of the issuer mirror.
 * Domain tag "CMPT_KEY_ROTATION_MIRROR_HOLDER_ONLY".
 * Decryption side is the holder's ConfidentialBalanceSpending (S1, S2) under
 * pk_H; the new issuer mirror (E1', E2') is created under pk_I'.
 * Ledger precondition, not proven here: ConfidentialBalanceInbox == EncZero.
 */
SECP256K1_API int
secp256k1_rotate_mirror_holder_prove(
    secp256k1_context const* ctx,
    unsigned char* proof_out,
    uint64_t balance,
    unsigned char const* sk_H,
    unsigned char const* r_new,
    secp256k1_pubkey const* pk_H,
    secp256k1_pubkey const* S1,
    secp256k1_pubkey const* S2,
    secp256k1_pubkey const* pk_I_new,
    secp256k1_pubkey const* E1_new,
    secp256k1_pubkey const* E2_new,
    unsigned char const* context_id);

SECP256K1_API int
secp256k1_rotate_mirror_holder_verify(
    secp256k1_context const* ctx,
    unsigned char const* proof,
    secp256k1_pubkey const* pk_H,
    secp256k1_pubkey const* S1,
    secp256k1_pubkey const* S2,
    secp256k1_pubkey const* pk_I_new,
    secp256k1_pubkey const* E1_new,
    secp256k1_pubkey const* E2_new,
    unsigned char const* context_id);

/**
 * pi_recbal - issuer completes holder key-loss recovery.
 * Domain tag "CMPT_KEY_ROTATION_RECOVER_BALANCE".
 * Decryption side is the issuer mirror (E1, E2) under the current pk_I; the new
 * spending balance (S1', S2') is created under the holder's registered
 * RecoveryKey pk_H'.
 * Ledger precondition, not proven here: the issuer mirror is already at the
 * current issuer key epoch.
 */
SECP256K1_API int
secp256k1_rotate_recover_balance_prove(
    secp256k1_context const* ctx,
    unsigned char* proof_out,
    uint64_t balance,
    unsigned char const* sk_I,
    unsigned char const* r_new,
    secp256k1_pubkey const* pk_I,
    secp256k1_pubkey const* E1,
    secp256k1_pubkey const* E2,
    secp256k1_pubkey const* pk_H_recovery,
    secp256k1_pubkey const* S1_new,
    secp256k1_pubkey const* S2_new,
    unsigned char const* context_id);

SECP256K1_API int
secp256k1_rotate_recover_balance_verify(
    secp256k1_context const* ctx,
    unsigned char const* proof,
    secp256k1_pubkey const* pk_I,
    secp256k1_pubkey const* E1,
    secp256k1_pubkey const* E2,
    secp256k1_pubkey const* pk_H_recovery,
    secp256k1_pubkey const* S1_new,
    secp256k1_pubkey const* S2_new,
    unsigned char const* context_id);

/** Compact proof size for the AND-composed "both mirrors" relation: 4 scalars. */
#define SECP256K1_ROTATE_BOTH_MIRRORS_PROOF_SIZE 128

/**
 * both-issuer - issuer-anchored AND-composed migration of both mirrors.
 * Domain tag "CMPT_KEY_ROTATION_MIRROR_ISSUER_AND_AUDITOR".
 * Decryption side is the current issuer mirror (E1, E2) under the current
 * pk_I; simultaneously re-encrypts to a new issuer mirror (E1', E2') under
 * pk_I' and a new auditor mirror (F1', F2') under pk_A', sharing one
 * randomness value across both. The verifier unconditionally checks
 * E1' == F1' (fix for Finding f:mhaud) -- this is NOT implied by the sigma
 * proof and must never be skipped.
 */
SECP256K1_API int
secp256k1_rotate_mirror_both_issuer_prove(
    secp256k1_context const* ctx,
    unsigned char* proof_out,
    uint64_t balance,
    unsigned char const* sk_I,
    unsigned char const* r_new,
    secp256k1_pubkey const* pk_I,
    secp256k1_pubkey const* E1,
    secp256k1_pubkey const* E2,
    secp256k1_pubkey const* pk_I_new,
    secp256k1_pubkey const* pk_A_new,
    secp256k1_pubkey const* E1_new,
    secp256k1_pubkey const* E2_new,
    secp256k1_pubkey const* F1_new,
    secp256k1_pubkey const* F2_new,
    unsigned char const* context_id);

SECP256K1_API int
secp256k1_rotate_mirror_both_issuer_verify(
    secp256k1_context const* ctx,
    unsigned char const* proof,
    secp256k1_pubkey const* pk_I,
    secp256k1_pubkey const* E1,
    secp256k1_pubkey const* E2,
    secp256k1_pubkey const* pk_I_new,
    secp256k1_pubkey const* pk_A_new,
    secp256k1_pubkey const* E1_new,
    secp256k1_pubkey const* E2_new,
    secp256k1_pubkey const* F1_new,
    secp256k1_pubkey const* F2_new,
    unsigned char const* context_id);

/**
 * both-holder - holder-anchored AND-composed migration of both mirrors.
 * Domain tag "CMPT_KEY_ROTATION_MIRROR_HOLDER_AND_AUDITOR".
 * Decryption side is the holder's ConfidentialBalanceSpending (S1, S2)
 * under pk_H; simultaneously re-encrypts to a new issuer mirror (E1', E2')
 * under pk_I' and a new auditor mirror (F1', F2') under pk_A', sharing one
 * randomness value across both. The verifier unconditionally checks
 * E1' == F1' (fix for Finding f:mhaud) -- this is NOT implied by the sigma
 * proof and must never be skipped.
 * Ledger precondition, not proven here: ConfidentialBalanceInbox == EncZero.
 */
SECP256K1_API int
secp256k1_rotate_mirror_both_holder_prove(
    secp256k1_context const* ctx,
    unsigned char* proof_out,
    uint64_t balance,
    unsigned char const* sk_H,
    unsigned char const* r_new,
    secp256k1_pubkey const* pk_H,
    secp256k1_pubkey const* S1,
    secp256k1_pubkey const* S2,
    secp256k1_pubkey const* pk_I_new,
    secp256k1_pubkey const* pk_A_new,
    secp256k1_pubkey const* E1_new,
    secp256k1_pubkey const* E2_new,
    secp256k1_pubkey const* F1_new,
    secp256k1_pubkey const* F2_new,
    unsigned char const* context_id);

SECP256K1_API int
secp256k1_rotate_mirror_both_holder_verify(
    secp256k1_context const* ctx,
    unsigned char const* proof,
    secp256k1_pubkey const* pk_H,
    secp256k1_pubkey const* S1,
    secp256k1_pubkey const* S2,
    secp256k1_pubkey const* pk_I_new,
    secp256k1_pubkey const* pk_A_new,
    secp256k1_pubkey const* E1_new,
    secp256k1_pubkey const* E2_new,
    secp256k1_pubkey const* F1_new,
    secp256k1_pubkey const* F2_new,
    unsigned char const* context_id);

/**
 * pi_ma - issuer-anchored migration of the auditor mirror.
 * Domain tag "CMPT_KEY_ROTATION_MIRROR_AUDITOR_ONLY".
 * Decryption side is the current issuer mirror (E1, E2) under the current
 * pk_I; the new auditor mirror (F1', F2') is created under pk_A'.  Covers
 * both auditor rotation (stale auditor mirror) and auditor late registration
 * (absent auditor mirror) -- the ledger-state precondition differs, the
 * relation does not. No previous-key field is required: the issuer does not
 * hold the auditor secret, so the balance necessarily comes from the issuer
 * mirror.
 */
SECP256K1_API int
secp256k1_rotate_mirror_auditor_prove(
    secp256k1_context const* ctx,
    unsigned char* proof_out,
    uint64_t balance,
    unsigned char const* sk_I,
    unsigned char const* r_new,
    secp256k1_pubkey const* pk_I,
    secp256k1_pubkey const* E1,
    secp256k1_pubkey const* E2,
    secp256k1_pubkey const* pk_A_new,
    secp256k1_pubkey const* F1_new,
    secp256k1_pubkey const* F2_new,
    unsigned char const* context_id);

SECP256K1_API int
secp256k1_rotate_mirror_auditor_verify(
    secp256k1_context const* ctx,
    unsigned char const* proof,
    secp256k1_pubkey const* pk_I,
    secp256k1_pubkey const* E1,
    secp256k1_pubkey const* E2,
    secp256k1_pubkey const* pk_A_new,
    secp256k1_pubkey const* F1_new,
    secp256k1_pubkey const* F2_new,
    unsigned char const* context_id);

/**
 * pi_mha - holder self-migration of the auditor mirror.
 * Domain tag "CMPT_KEY_ROTATION_MIRROR_HOLDER_AUDITOR_ONLY".
 * Decryption side is the holder's ConfidentialBalanceSpending (S1, S2) under
 * pk_H; the new auditor mirror (F1', F2') is created under pk_A'.
 * Structurally identical to pi_mh with the auditor mirror as re-encryption
 * target instead of the issuer mirror; the distinct domain tag keeps the two
 * proofs from being interchangeable.
 * Ledger precondition, not proven here: ConfidentialBalanceInbox == EncZero.
 */
SECP256K1_API int
secp256k1_rotate_mirror_holder_auditor_prove(
    secp256k1_context const* ctx,
    unsigned char* proof_out,
    uint64_t balance,
    unsigned char const* sk_H,
    unsigned char const* r_new,
    secp256k1_pubkey const* pk_H,
    secp256k1_pubkey const* S1,
    secp256k1_pubkey const* S2,
    secp256k1_pubkey const* pk_A_new,
    secp256k1_pubkey const* F1_new,
    secp256k1_pubkey const* F2_new,
    unsigned char const* context_id);

SECP256K1_API int
secp256k1_rotate_mirror_holder_auditor_verify(
    secp256k1_context const* ctx,
    unsigned char const* proof,
    secp256k1_pubkey const* pk_H,
    secp256k1_pubkey const* S1,
    secp256k1_pubkey const* S2,
    secp256k1_pubkey const* pk_A_new,
    secp256k1_pubkey const* F1_new,
    secp256k1_pubkey const* F2_new,
    unsigned char const* context_id);

/** Compact proof size for the holder key-rotation relation: 7 scalars. */
#define SECP256K1_ROTATE_HOLDER_ROTATE_PROOF_SIZE 224

/**
 * pi_hr - holder voluntary key rotation (spending balance + inbox).
 * Domain tag "CMPT_KEY_ROTATION_HOLDER_ROTATE".
 * Not an instantiation of the shared re-encryption core: it has six nonces
 * (not four) and an extra PoK conjunct for the newly rotated-to key pk_H',
 * plus a second re-encryption branch binding the new inbox ciphertext
 * (I1', I2') under pk_H' -- the branch the proof specification originally
 * omitted (a required ledger write left unproven, later folded into the
 * spending balance by MergeInbox). sk_H is shared between the CBS and CBIN
 * decryption branches; pk_H' is shared as the re-encryption target of both.
 * Witness (b, sk_H, r_s', sk_H', b_in, r_i') in Z_q^6.
 */
SECP256K1_API int
secp256k1_rotate_holder_rotate_prove(
    secp256k1_context const* ctx,
    unsigned char* proof_out,
    uint64_t balance,
    uint64_t inbox_balance,
    unsigned char const* sk_H,
    unsigned char const* r_s_new,
    unsigned char const* sk_H_new,
    unsigned char const* r_i_new,
    secp256k1_pubkey const* pk_H,
    secp256k1_pubkey const* pk_H_new,
    secp256k1_pubkey const* S1,
    secp256k1_pubkey const* S2,
    secp256k1_pubkey const* S1_new,
    secp256k1_pubkey const* S2_new,
    secp256k1_pubkey const* I1,
    secp256k1_pubkey const* I2,
    secp256k1_pubkey const* I1_new,
    secp256k1_pubkey const* I2_new,
    unsigned char const* context_id);

SECP256K1_API int
secp256k1_rotate_holder_rotate_verify(
    secp256k1_context const* ctx,
    unsigned char const* proof,
    secp256k1_pubkey const* pk_H,
    secp256k1_pubkey const* pk_H_new,
    secp256k1_pubkey const* S1,
    secp256k1_pubkey const* S2,
    secp256k1_pubkey const* S1_new,
    secp256k1_pubkey const* S2_new,
    secp256k1_pubkey const* I1,
    secp256k1_pubkey const* I2,
    secp256k1_pubkey const* I1_new,
    secp256k1_pubkey const* I2_new,
    unsigned char const* context_id);

/** Compact proof size for the recovery key possession relation: 2 scalars. */
#define SECP256K1_ROTATE_RECOVERY_KEY_PROOF_SIZE 64

/**
 * pi_rec - holder registers a RecoveryKey after losing sk_H.
 * Domain tag "CMPT_KEY_ROTATION_HOLDER_RECOVERY".
 * A bare Schnorr proof of possession of sk_H' for the newly registered
 * RecoveryKey pk_H'; it touches no balance and moves no funds.  The balance is
 * migrated later by the issuer via secp256k1_rotate_recover_balance_*.
 * Same relation as secp256k1_mpt_pok_sk_*, separated only by domain tag, so a
 * registration proof cannot be replayed as a recovery-key proof or vice versa.
 * Argument order is (sk, pk), consistent with the other secp256k1_rotate_*_prove
 * functions, and the reverse of secp256k1_mpt_pok_sk_prove, which takes (pk, sk).
 */
SECP256K1_API int
secp256k1_rotate_recovery_key_prove(
    secp256k1_context const* ctx,
    unsigned char* proof_out,
    unsigned char const* sk_H_recovery,
    secp256k1_pubkey const* pk_H_recovery,
    unsigned char const* context_id);

SECP256K1_API int
secp256k1_rotate_recovery_key_verify(
    secp256k1_context const* ctx,
    unsigned char const* proof,
    secp256k1_pubkey const* pk_H_recovery,
    unsigned char const* context_id);

#ifdef __cplusplus
}
#endif

#endif  // SECP256K1_MPT_H
