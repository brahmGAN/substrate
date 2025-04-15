// This file is part of Substrate.

// Copyright (C) 2023 Parity Technologies (UK) Ltd.
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

//! Tests for the minting functionality of the Balances pallet.

use super::*;
use frame_support::traits::Currency;

#[test]
fn mint_tokens_to_sudo_works_for_investors() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build_and_execute_with(|| {
			// Use account ID 1 as the sudo account
			let sudo_account: u64 = 1;
			
			// Initial balance of sudo account and total issuance
			let initial_balance = Balances::free_balance(sudo_account);
			let initial_issuance = Balances::total_issuance();
			
			// Mint 1M tokens for investors
			let mint_amount = 1_000_000;
			assert_ok!(Balances::mint_tokens_to_sudo(RawOrigin::Root.into(), sudo_account, mint_amount, false));
			
			// Check sudo balance increased
			assert_eq!(Balances::free_balance(sudo_account), initial_balance + mint_amount);
			
			// Check total issuance increased
			assert_eq!(Balances::total_issuance(), initial_issuance + mint_amount);
			
			// Check minted tokens storage was updated correctly
			let (investor_minted, liquidity_minted) = Balances::minted_tokens();
			assert_eq!(investor_minted, mint_amount);
			assert_eq!(liquidity_minted, 0);
			
			// Check events were emitted
			System::assert_has_event(RuntimeEvent::Balances(crate::Event::Minted {
				who: sudo_account,
				amount: mint_amount,
			}));
			
			System::assert_has_event(RuntimeEvent::Balances(crate::Event::TotalIssuanceUpdated {
				current: initial_issuance,
				actualised: initial_issuance + mint_amount,
			}));
		});
}

#[test]
fn mint_tokens_to_sudo_works_for_liquidity() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build_and_execute_with(|| {
			// Use account ID 1 as the sudo account
			let sudo_account: u64 = 1;
			
			// Initial balance of sudo account and total issuance
			let initial_balance = Balances::free_balance(sudo_account);
			let initial_issuance = Balances::total_issuance();
			
			// Mint 1M tokens for liquidity
			let mint_amount = 1_000_000;
			assert_ok!(Balances::mint_tokens_to_sudo(RawOrigin::Root.into(), sudo_account, mint_amount, true));
			
			// Check sudo balance increased
			assert_eq!(Balances::free_balance(sudo_account), initial_balance + mint_amount);
			
			// Check total issuance increased
			assert_eq!(Balances::total_issuance(), initial_issuance + mint_amount);
			
			// Check minted tokens storage was updated correctly
			let (investor_minted, liquidity_minted) = Balances::minted_tokens();
			assert_eq!(investor_minted, 0);
			assert_eq!(liquidity_minted, mint_amount);
		});
}

#[test]
fn mint_tokens_investor_exceeds_cap() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build_and_execute_with(|| {
			// Use account ID 1 as the sudo account
			let sudo_account: u64 = 1;
			
			// Define the maximum mintable tokens (20M for investors)
			const MAX_INVESTOR_TOKENS: u64 = 20_000_000;
			
			// Try to mint more than cap for investors
			assert_noop!(
				Balances::mint_tokens_to_sudo(RawOrigin::Root.into(), sudo_account, MAX_INVESTOR_TOKENS + 1, false),
				Error::<Test>::MintCapExceeded
			);
		});
}

#[test]
fn mint_tokens_liquidity_exceeds_cap() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build_and_execute_with(|| {
			// Use account ID 1 as the sudo account
			let sudo_account: u64 = 1;
			
			// Define the maximum mintable tokens (20M for liquidity)
			const MAX_LIQUIDITY_TOKENS: u64 = 20_000_000;
			
			// Try to mint more than cap for liquidity
			assert_noop!(
				Balances::mint_tokens_to_sudo(RawOrigin::Root.into(), sudo_account, MAX_LIQUIDITY_TOKENS + 1, true),
				Error::<Test>::MintCapExceeded
			);
		});
}

#[test]
fn mint_tokens_non_root_fails() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build_and_execute_with(|| {
			// Use account ID 1 as the sudo account
			let sudo_account: u64 = 1;
			
			// Use account ID 2 as a regular account
			let non_root: u64 = 2;
			
			// Try to mint tokens with non-root origin
			let mint_amount = 1_000_000;
			assert_noop!(
				Balances::mint_tokens_to_sudo(RuntimeOrigin::signed(non_root), sudo_account, mint_amount, false),
				DispatchError::BadOrigin
			);
		});
}

#[test]
fn mint_tokens_incremental_up_to_cap() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build_and_execute_with(|| {
			// Use account ID 1 as the sudo account
			let sudo_account: u64 = 1;
			
			// Mint tokens incrementally for investors up to the cap
			let mint_amount = 10_000_000; // 10M each time
			
			// First mint
			assert_ok!(Balances::mint_tokens_to_sudo(RawOrigin::Root.into(), sudo_account, mint_amount, false));
			let (investor_minted, _) = Balances::minted_tokens();
			assert_eq!(investor_minted, mint_amount);
			
			// Second mint
			assert_ok!(Balances::mint_tokens_to_sudo(RawOrigin::Root.into(), sudo_account, mint_amount, false));
			let (investor_minted, _) = Balances::minted_tokens();
			assert_eq!(investor_minted, mint_amount * 2);
			
			// Third mint should fail (10M + 10M + 10M > 20M)
			assert_noop!(
				Balances::mint_tokens_to_sudo(RawOrigin::Root.into(), sudo_account, mint_amount, false),
				Error::<Test>::MintCapExceeded
			);
			
			// But we can still mint up to the cap (20M - 20M = 0)
			assert_noop!(
				Balances::mint_tokens_to_sudo(RawOrigin::Root.into(), sudo_account, 1, false),
				Error::<Test>::MintCapExceeded
			);
		});
}

#[test]
fn mint_tokens_separate_caps() {
	ExtBuilder::default()
		.existential_deposit(1)
		.monied(true)
		.build_and_execute_with(|| {
			// Use account ID 1 as the sudo account
			let sudo_account: u64 = 1;
			
			// Define caps
			const MAX_INVESTOR_TOKENS: u64 = 20_000_000;
			const MAX_LIQUIDITY_TOKENS: u64 = 20_000_000;
			
			// Mint full amount for investors
			assert_ok!(Balances::mint_tokens_to_sudo(RawOrigin::Root.into(), sudo_account, MAX_INVESTOR_TOKENS, false));
			
			// Verify investors cap is reached
			let (investor_minted, _) = Balances::minted_tokens();
			assert_eq!(investor_minted, MAX_INVESTOR_TOKENS);
			
			// Cannot mint more for investors
			assert_noop!(
				Balances::mint_tokens_to_sudo(RawOrigin::Root.into(), sudo_account, 1, false),
				Error::<Test>::MintCapExceeded
			);
			
			// But can still mint for liquidity
			assert_ok!(Balances::mint_tokens_to_sudo(RawOrigin::Root.into(), sudo_account, MAX_LIQUIDITY_TOKENS, true));
			
			// Verify both caps are now reached
			let (investor_minted, liquidity_minted) = Balances::minted_tokens();
			assert_eq!(investor_minted, MAX_INVESTOR_TOKENS);
			assert_eq!(liquidity_minted, MAX_LIQUIDITY_TOKENS);
			
			// Cannot mint more for liquidity
			assert_noop!(
				Balances::mint_tokens_to_sudo(RawOrigin::Root.into(), sudo_account, 1, true),
				Error::<Test>::MintCapExceeded
			);
			
			// Get initial issuance
			let initial_issuance = Balances::total_issuance() - MAX_INVESTOR_TOKENS - MAX_LIQUIDITY_TOKENS;
			
			// Total issuance should reflect both minted amounts
			assert_eq!(Balances::total_issuance(), initial_issuance + MAX_INVESTOR_TOKENS + MAX_LIQUIDITY_TOKENS);
		});
}