/**
 * Onboarding Feature Module
 *
 * Handles first-time setup:
 * 1. Introduction to ECHO privacy boundaries
 * 2. Selection of local documents folder
 * 3. Optional AI provider configuration
 */

export interface OnboardingState {
  hasCompletedOnboarding: boolean;
  step: number;
  totalSteps: number;
}

export const INITIAL_ONBOARDING_STATE: OnboardingState = {
  hasCompletedOnboarding: false,
  step: 1,
  totalSteps: 3,
};

