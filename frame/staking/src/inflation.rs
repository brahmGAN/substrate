// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module expose one function `P_NPoS` (Payout NPoS) or `compute_total_payout` which returns
//! the total payout for the era given the era duration and the staking rate in NPoS.
//! The staking rate in NPoS is the total amount of tokens staked by nominators and validators,
//! divided by the total token supply.

use sp_runtime::{curve::PiecewiseLinear, traits::AtLeast32BitUnsigned, Perbill};

/// The total payout to all validators (and their nominators) per era and maximum payout.
///
/// Defined as such:
/// `staker-payout = yearly_inflation(npos_token_staked / total_tokens) * total_tokens /
/// era_per_year` `maximum-payout = max_yearly_inflation * total_tokens / era_per_year`
///
/// `era_duration` is expressed in millisecond.
use core::ops::{Div, Mul};

pub fn compute_total_payout<N>(
    _yearly_inflation: &PiecewiseLinear<'static>,
    circulating_supply: N,
    total_supply: N,
    _era_duration: u64,
) -> (N, N)
where
    N: AtLeast32BitUnsigned + Clone + core::convert::From<u128> + Div<Output = N> + Mul<Output = N> + PartialOrd,
{
    // Define base emission rate (B₀) = 1440 tokens/day (in wei units)
    let wei_multiplier = N::from(1000000000000000000u128);
    let b0 = N::from(1440u128) * wei_multiplier;
    
    // Calculate half of total supply
    let half_supply = total_supply.clone() / N::from(2u128);
    
    // Calculate total emission based on formula D(Sc)
    let total_emission: N;
    
    if circulating_supply <= half_supply {
        // D(Sc) = B₀ × 2ⁿ when Sc ≤ 0.5 · St
        
        // Calculate n based on specific milestone percentages from the table:
        // 3.13%, 6.25%, 12.5%, 25%, 50%
        let mut n = 0;
        
        // 3.13% milestone
        if circulating_supply >= (total_supply.clone() * N::from(313u128) / N::from(10000u128)) {
            n = 1; // 2880 emission (2¹ × base rate)
        }
        
        // 6.25% milestone
        if circulating_supply >= (total_supply.clone() * N::from(625u128) / N::from(10000u128)) {
            n = 2; // 5760 emission (2² × base rate)
        }
        
        // 12.5% milestone
        if circulating_supply >= (total_supply.clone() * N::from(1250u128) / N::from(10000u128)) {
            n = 3; // 11520 emission (2³ × base rate)
        }
        
        // 25% milestone
        if circulating_supply >= (total_supply.clone() * N::from(2500u128) / N::from(10000u128)) {
            n = 4; // 23040 emission (2⁴ × base rate)
        }
        
        // 50% milestone
        if circulating_supply >= (total_supply.clone() * N::from(5000u128) / N::from(10000u128)) {
            n = 5; // 46080 emission (2⁵ × base rate)
        }
        
        // Calculate 2ⁿ
        let two_power_n = N::from(1u128 << n);
        
        // Apply the formula D(Sc) = B₀ × 2ⁿ
        total_emission = b0 * two_power_n;
    } else {
        // Apply new formula: D(Sc) = B₀ × 1/2^⌈log₂(St/(St-Sc))⌉
        
        // Calculate St/(St-Sc)
        let remaining = total_supply.clone() - circulating_supply.clone();
        
        // Ensure we don't divide by zero or get an unreasonably small remaining amount
        // By setting a minimum safe remaining amount, we cap the maximum divisor
        let min_remaining = total_supply.clone() / N::from(1_000_000u128); // 0.0001% of total
        let safe_remaining = if remaining < min_remaining {
            min_remaining
        } else {
            remaining
        };
        
        // Calculate the ratio St/(St-Sc) with safety cap
        let ratio = total_supply.clone() / safe_remaining;
        
        // Convert ratio to u128 for bit manipulation, with a safe maximum
        let max_u128 = u128::MAX / 2; // Avoid overflows
        let ratio_u128 = ratio.clone().try_into().unwrap_or(max_u128);
        
        // Find the position of the highest bit set (floor of log₂)
        let log2_floor = 128u32.saturating_sub(ratio_u128.leading_zeros()).saturating_sub(1);
        
        // Calculate ceiling of log₂(ratio)
        let log2_ceiling = if ratio_u128 == (1u128 << log2_floor) {
            log2_floor
        } else {
            log2_floor + 1
        };
        
        // Cap the maximum divisor to prevent emission from becoming too small
        let max_log2 = 120u32; // Allow very small emissions but not zero
        let capped_log2 = core::cmp::min(log2_ceiling, max_log2);
        
        // Calculate 2^⌈log₂(ratio)⌉ with safety cap
        let divisor = N::from(1u128 << capped_log2);
        
        // Apply the formula D(Sc) = B₀ × 1/2^⌈log2(St/(St-Sc))⌉
        total_emission = b0 / divisor;
    }
    
    // Validator payout is 1/4 of total emission
    let validator_payout = total_emission.clone() * N::from(1u128) / N::from(4u128);
    
    (validator_payout, total_emission)
}
mod tests {
    use super::*;
    #[test]
    fn test_new_emission_formula() {
        // Create an empty PiecewiseLinear directly (no constructor needed)
        let dummy_inflation = PiecewiseLinear { 
            points: &[],
            maximum: Perbill::zero(),
        };
        
        // Test with 200M total tokens
        let total_tokens = 200_000_000_000_000_000_000_000_000u128; // 200M tokens in wei
        
        // Base emission rate
        let base_rate = 1440_000_000_000_000_000_000u128;
        
        // First test the 50% boundary case
        // At exactly 50%, we should still be using the old formula with n=5 (2^5 * base_rate)
        let half_supply = total_tokens / 2;
        let (_, emission_at_half) = compute_total_payout::<u128>(&dummy_inflation, half_supply, total_tokens, 0);
        let expected_at_half = base_rate * 32; // 2^5 * base_rate
        assert_eq!(
            emission_at_half,
            expected_at_half,
            "At exactly 50%, emission should be 2^5 * base_rate"
        );
        
        // Now test percentages beyond 50% using the new formula
        let test_percentages = [
            // percentage (in basis points), expected divisor (2^⌈log₂(St/(St-Sc))⌉)
            (5001, 2),     // 50.01%: St/(St-Sc) = 2.0004, log₂ ceiling = 1, 2^1 = 2
            (7500, 4),     // 75%: St/(St-Sc) = 4, log₂(4) = 2, 2^2 = 4
            (8750, 8),     // 87.5%: St/(St-Sc) = 8, log₂(8) = 3, 2^3 = 8
            (9375, 16),    // 93.75%: St/(St-Sc) = 16, log₂(16) = 4, 2^4 = 16
            (9688, 32),    // 96.88%: St/(St-Sc) = 32, log₂(32) = 5, 2^5 = 32
            (9844, 64),    // 98.44%: St/(St-Sc) = 64, log₂(64) = 6, 2^6 = 64
            (9922, 128),   // 99.22%: St/(St-Sc) = 128, log₂(128) = 7, 2^7 = 128
        ];
        
        for (basis_points, expected_divisor) in test_percentages {
            // Calculate circulating supply at this percentage (in basis points, e.g. 7500 = 75%)
            let circulating_supply = total_tokens * basis_points / 10000;
            
            // Get emission at this supply level
            let (validator_payout, total_emission) = 
                compute_total_payout::<u128>(&dummy_inflation, circulating_supply, total_tokens, 0);
                
            // Expected emission is base_rate / expected_divisor
            let expected_emission = base_rate / expected_divisor;
            
            // Verify emission matches expected value
            assert_eq!(
                total_emission, 
                expected_emission,
                "Emission at {}.{}% should be base rate / {}", 
                basis_points / 100,
                basis_points % 100,
                expected_divisor
            );
            
            // Verify validator payout is 1/4 of total emission
            assert_eq!(validator_payout * 4, total_emission);
        }
        
        // Test that the emission never becomes zero even when circulating supply is very close to total
        let near_total_supply = total_tokens - 1u128;
        let (_, emission_near_total) = 
            compute_total_payout::<u128>(&dummy_inflation, near_total_supply, total_tokens, 0);
        
        assert!(emission_near_total > 0, "Emission should never be zero");
        }
    }