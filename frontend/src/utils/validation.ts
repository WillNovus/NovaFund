import { ProjectFormData, ValidationErrors } from '@/types/project';

export const validateBasics = (data: ProjectFormData): ValidationErrors => {
  const errors: ValidationErrors = {};
  
  if (!data.title.trim()) {
    errors.title = 'Project title is required';
  } else if (data.title.length < 3) {
    errors.title = 'Title must be at least 3 characters';
  } else if (data.title.length > 100) {
    errors.title = 'Title must not exceed 100 characters';
  }
  
  if (!data.description.trim()) {
    errors.description = 'Project description is required';
  } else if (data.description.length < 20) {
    errors.description = 'Description must be at least 20 characters';
  } else if (data.description.length > 2000) {
    errors.description = 'Description must not exceed 2000 characters';
  }
  
  if (!data.category) {
    errors.category = 'Please select a category';
  }
  
  return errors;
};

export const validateFunding = (data: ProjectFormData): ValidationErrors => {
  const errors: ValidationErrors = {};
  
  if (!data.fundingGoal || data.fundingGoal <= 0) {
    errors.fundingGoal = 'Funding goal must be greater than 0';
  } else if (data.fundingGoal > 1000000000) {
    errors.fundingGoal = 'Funding goal is too large';
  }
  
  if (!data.duration || data.duration <= 0) {
    errors.duration = 'Duration must be greater than 0 days';
  } else if (data.duration > 365) {
    errors.duration = 'Duration cannot exceed 365 days';
  }
  
  if (!data.walletAddress.trim()) {
    errors.walletAddress = 'Stellar wallet address is required';
  } else if (!isValidStellarAddress(data.walletAddress)) {
    errors.walletAddress = 'Please enter a valid Stellar address';
  }
  
  return errors;
};

export const validateMilestones = (data: ProjectFormData): ValidationErrors => {
  const errors: ValidationErrors = {};
  
  if (data.milestones.length === 0) {
    errors.milestones = 'At least one milestone is required';
  } else {
    const totalPercentage = data.milestones.reduce((sum, m) => sum + m.percentage, 0);
    
    if (totalPercentage !== 100) {
      errors.milestones = `Milestone percentages must total 100% (currently ${totalPercentage}%)`;
    }
    
    data.milestones.forEach((milestone, index) => {
      if (!milestone.title.trim()) {
        errors[`milestone_${index}_title`] = 'Milestone title is required';
      }
      if (milestone.percentage <= 0) {
        errors[`milestone_${index}_percentage`] = 'Percentage must be greater than 0';
      }
    });
  }
  
  return errors;
};

export const validateAllSteps = (data: ProjectFormData): ValidationErrors => {
  return {
    ...validateBasics(data),
    ...validateFunding(data),
    ...validateMilestones(data)
  };
};

// Simple Stellar address validation (starts with G, 56 characters)
const isValidStellarAddress = (address: string): boolean => {
  const stellarRegex = /^G[A-Z2-7]{55}$/;
  return stellarRegex.test(address);
};

export const formatCurrency = (amount: number): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 0,
    maximumFractionDigits: 0
  }).format(amount);
};
