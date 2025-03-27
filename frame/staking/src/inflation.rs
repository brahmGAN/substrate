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
        // D(Sc) = B₀ × 1/2ᵐ when Sc > 0.5 · St
        
        // Calculate m based on specific milestone percentages from the table:
        // 75%, 87.5%, 93.75%, 96.88%
        let mut m = 0;
        
        // 75% milestone
        if circulating_supply >= (total_supply.clone() * N::from(7500u128) / N::from(10000u128)) {
            m = 1; // 23040 emission (base rate × 2⁵/2¹)
        }
        
        // 87.5% milestone
        if circulating_supply >= (total_supply.clone() * N::from(8750u128) / N::from(10000u128)) {
            m = 2; // 11520 emission (base rate × 2⁵/2²)
        }
        
        // 93.75% milestone
        if circulating_supply >= (total_supply.clone() * N::from(9375u128) / N::from(10000u128)) {
            m = 3; // 5760 emission (base rate × 2⁵/2³)
        }
        
        // 96.88% milestone
        if circulating_supply >= (total_supply.clone() * N::from(9688u128) / N::from(10000u128)) {
            m = 4; // 2880 emission (base rate × 2⁵/2⁴)
        }
        
        // Calculate 2⁵/2ᵐ (max rate divided by 2ᵐ)
        let max_multiplier = N::from(1u128 << 5); // 2⁵
        let divisor = N::from(1u128 << m); // 2ᵐ
        
        // Apply the formula D(Sc) = B₀ × 2⁵/2ᵐ = B₀ × 2⁵⁻ᵐ
        total_emission = b0 * max_multiplier / divisor;
    }
    
    // Validator payout is 480/1440 (1/3) of total emission
    let validator_payout = total_emission.clone() * N::from(480u128) / N::from(1440u128);
    
    (validator_payout, total_emission)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_emission_at_specific_milestones() {
        // Create an empty PiecewiseLinear directly (no constructor needed)
        let dummy_inflation = PiecewiseLinear { 
            points: &[],
            maximum: Perbill::zero(),
        };
        
        // Test with 200M total tokens
        let total_tokens = 200_000_000_000_000_000_000_000_000u128; // 200M tokens in wei
        
        // Base emission rate
        let base_rate = 1440_000_000_000_000_000_000u128;
        
        // Test at each milestone from the table
        let milestones = [
            (313, 2), // 3.13% -> 2880 (2¹ × base)
            (625, 4), // 6.25% -> 5760 (2² × base)
            (1250, 8), // 12.5% -> 11520 (2³ × base)
            (2500, 16), // 25% -> 23040 (2⁴ × base)
            (5000, 32), // 50% -> 46080 (2⁵ × base)
            (7500, 16), // 75% -> 23040 (2⁵/2¹ × base)
            (8750, 8), // 87.5% -> 11520 (2⁵/2² × base)
            (9375, 4), // 93.75% -> 5760 (2⁵/2³ × base)
            (9688, 2), // 96.88% -> 2880 (2⁵/2⁴ × base)
        ];
        
        for (percent_x100, multiplier) in milestones {
            // Calculate circulating supply at this percentage
            let circulating_supply = total_tokens * percent_x100 / 10000;
            
            // Get emission at this supply level
            let (validator_payout, total_emission) = 
                compute_total_payout::<u128>(&dummy_inflation, circulating_supply, total_tokens, 0);
                
            // Expected emission is base_rate × multiplier
            let expected_emission = base_rate * multiplier;
            
            // Verify emission matches table
            assert_eq!(
                total_emission, 
                expected_emission,
                "Emission at {}.{}% should be {} × base rate", 
                percent_x100 / 100, 
                percent_x100 % 100,
                multiplier
            );
            
            // Verify validator payout is 1/3 of total emission
            assert_eq!(validator_payout * 3, total_emission);
        }
    }
}