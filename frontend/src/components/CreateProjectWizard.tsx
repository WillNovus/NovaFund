'use client';

import { useState, useEffect } from 'react';
import { ProjectFormData, INITIAL_PROJECT_DATA, ValidationErrors } from '@/types/project';
import { validateBasics, validateFunding, validateMilestones, validateAllSteps } from '@/utils/validation';
import BasicsStep from './steps/BasicsStep';
import FundingStep from './steps/FundingStep';
import MilestonesStep from './steps/MilestonesStep';
import ReviewStep from './steps/ReviewStep';
import PreviewCard from './PreviewCard';
import { ChevronLeft, ChevronRight, Check, Loader2 } from 'lucide-react';

const STEPS = [
  { id: 0, title: 'Basics', description: 'Project information' },
  { id: 1, title: 'Funding', description: 'Financial details' },
  { id: 2, title: 'Milestones', description: 'Project roadmap' },
  { id: 3, title: 'Review', description: 'Final check' }
];

export default function CreateProjectWizard() {
  const [currentStep, setCurrentStep] = useState(0);
  const [formData, setFormData] = useState<ProjectFormData>(INITIAL_PROJECT_DATA);
  const [errors, setErrors] = useState<ValidationErrors>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [visitedSteps, setVisitedSteps] = useState<Set<number>>(new Set([0]));
  const [isInitialized, setIsInitialized] = useState(false);

  // Load persisted data from localStorage on mount
  useEffect(() => {
    if (typeof window === 'undefined') return;
    
    try {
      const savedData = localStorage.getItem('projectFormData');
      const savedStep = localStorage.getItem('projectCurrentStep');
      const savedVisitedSteps = localStorage.getItem('projectVisitedSteps');
      
      if (savedData) {
        try {
          const parsed = JSON.parse(savedData);
          // Merge with initial data to ensure all fields exist
          const mergedData = { ...INITIAL_PROJECT_DATA, ...parsed };
          // Ensure milestones is an array
          if (!Array.isArray(mergedData.milestones)) {
            mergedData.milestones = [];
          }
          setFormData(mergedData);
        } catch (e) {
          console.error('Failed to parse saved form data', e);
        }
      }
      
      if (savedStep) {
        try {
          const step = parseInt(savedStep, 10);
          if (!isNaN(step) && step >= 0 && step < STEPS.length) {
            setCurrentStep(step);
          }
        } catch (e) {
          console.error('Failed to parse saved step', e);
        }
      }
      
      if (savedVisitedSteps) {
        try {
          const visited = JSON.parse(savedVisitedSteps);
          if (Array.isArray(visited)) {
            setVisitedSteps(new Set(visited));
          }
        } catch (e) {
          console.error('Failed to parse saved visited steps', e);
        }
      }
    } catch (e) {
      console.error('Error accessing localStorage', e);
    } finally {
      setIsInitialized(true);
    }
  }, []);

  // Persist form data to localStorage (only after initialization)
  useEffect(() => {
    if (!isInitialized || typeof window === 'undefined') return;
    try {
      localStorage.setItem('projectFormData', JSON.stringify(formData));
    } catch (e) {
      console.error('Failed to save form data to localStorage', e);
    }
  }, [formData, isInitialized]);

  // Persist current step to localStorage (only after initialization)
  useEffect(() => {
    if (!isInitialized || typeof window === 'undefined') return;
    try {
      localStorage.setItem('projectCurrentStep', currentStep.toString());
    } catch (e) {
      console.error('Failed to save current step to localStorage', e);
    }
  }, [currentStep, isInitialized]);

  // Persist visited steps to localStorage (only after initialization)
  useEffect(() => {
    if (!isInitialized || typeof window === 'undefined') return;
    try {
      localStorage.setItem('projectVisitedSteps', JSON.stringify(Array.from(visitedSteps)));
    } catch (e) {
      console.error('Failed to save visited steps to localStorage', e);
    }
  }, [visitedSteps, isInitialized]);

  const updateField = (field: keyof ProjectFormData, value: any) => {
    setFormData(prev => ({ ...prev, [field]: value }));
    // Clear error for this field when user starts typing
    if (errors[field]) {
      setErrors(prev => {
        const newErrors = { ...prev };
        delete newErrors[field];
        return newErrors;
      });
    }
  };

  const validateCurrentStep = (): boolean => {
    let stepErrors: ValidationErrors = {};
    
    switch (currentStep) {
      case 0:
        stepErrors = validateBasics(formData);
        break;
      case 1:
        stepErrors = validateFunding(formData);
        break;
      case 2:
        stepErrors = validateMilestones(formData);
        break;
      case 3:
        stepErrors = validateAllSteps(formData);
        break;
    }

    setErrors(stepErrors);
    return Object.keys(stepErrors).length === 0;
  };

  const goToStep = (step: number) => {
    setCurrentStep(step);
    setVisitedSteps(prev => {
      const newSet = new Set(prev);
      newSet.add(step);
      return newSet;
    });
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };

  const handleNext = () => {
    if (validateCurrentStep()) {
      if (currentStep < STEPS.length - 1) {
        goToStep(currentStep + 1);
      }
    }
  };

  const handleBack = () => {
    if (currentStep > 0) {
      goToStep(currentStep - 1);
    }
  };

  const handleSubmit = async () => {
    if (!validateCurrentStep()) {
      return;
    }

    setIsSubmitting(true);
    
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      console.log('Submitting project:', formData);
      
      // Clear saved data on successful submission
      localStorage.removeItem('projectFormData');
      localStorage.removeItem('projectCurrentStep');
      localStorage.removeItem('projectVisitedSteps');
      
      // Show success message or redirect
      alert('Project created successfully! 🎉');
      
      // Reset form
      setFormData(INITIAL_PROJECT_DATA);
      setCurrentStep(0);
      setVisitedSteps(new Set([0]));
      
    } catch (error) {
      console.error('Failed to create project:', error);
      alert('Failed to create project. Please try again.');
    } finally {
      setIsSubmitting(false);
    }
  };

  const renderStep = () => {
    const stepProps = {
      data: formData,
      errors,
      onChange: updateField
    };

    switch (currentStep) {
      case 0:
        return <BasicsStep {...stepProps} />;
      case 1:
        return <FundingStep {...stepProps} />;
      case 2:
        return <MilestonesStep {...stepProps} />;
      case 3:
        return <ReviewStep data={formData} onEdit={goToStep} />;
      default:
        return null;
    }
  };

  const isStepComplete = (stepIndex: number): boolean => {
    switch (stepIndex) {
      case 0:
        return Object.keys(validateBasics(formData)).length === 0;
      case 1:
        return Object.keys(validateFunding(formData)).length === 0;
      case 2:
        return Object.keys(validateMilestones(formData)).length === 0;
      case 3:
        return Object.keys(validateAllSteps(formData)).length === 0;
      default:
        return false;
    }
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="bg-card border-b border-border sticky top-0 z-10 shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-foreground">Create Project</h1>
              <p className="text-sm text-muted-foreground mt-1">
                Step {currentStep + 1} of {STEPS.length}
              </p>
            </div>
            <button
              type="button"
              onClick={() => {
                if (confirm('Are you sure you want to exit? Your progress will be saved.')) {
                  window.location.href = '/';
                }
              }}
              className="text-muted-foreground hover:text-foreground text-sm font-medium hover:bg-accent px-4 py-2 rounded-md transition-colors"
            >
              Exit
            </button>
          </div>
        </div>
      </header>

      {/* Progress Bar */}
      <div className="bg-card border-b border-border">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <div className="flex items-center justify-between">
            {STEPS.map((step, index) => (
              <div key={step.id} className="flex-1 flex items-center">
                <div className="flex flex-col items-center flex-1">
                  {/* Step Circle */}
                  <button
                    type="button"
                    onClick={() => visitedSteps.has(index) && goToStep(index)}
                    disabled={!visitedSteps.has(index)}
                    className={`w-10 h-10 rounded-full flex items-center justify-center font-semibold transition-all ${
                      currentStep === index
                        ? 'bg-primary text-primary-foreground ring-4 ring-primary/20 shadow-lg'
                        : isStepComplete(index)
                        ? 'bg-primary text-primary-foreground shadow-md'
                        : visitedSteps.has(index)
                        ? 'bg-secondary text-secondary-foreground hover:bg-secondary/80 cursor-pointer'
                        : 'bg-muted text-muted-foreground'
                    }`}
                  >
                    {isStepComplete(index) && currentStep !== index ? (
                      <Check className="w-5 h-5" />
                    ) : (
                      index + 1
                    )}
                  </button>
                  {/* Step Label */}
                  <div className="mt-2 text-center">
                    <p className={`text-sm font-medium ${
                      currentStep === index ? 'text-foreground' : 'text-muted-foreground'
                    }`}>
                      {step.title}
                    </p>
                    <p className="text-xs text-muted-foreground hidden sm:block">{step.description}</p>
                  </div>
                </div>
                {/* Connector Line */}
                {index < STEPS.length - 1 && (
                  <div className={`h-1 flex-1 mx-2 mt-[-2.5rem] rounded-full ${
                    isStepComplete(index) ? 'bg-primary' : 'bg-border'
                  }`} />
                )}
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Form Area */}
          <div className="lg:col-span-2">
            <div className="bg-card rounded-lg shadow-md border border-border p-6 sm:p-8">
              {renderStep()}

              {/* Navigation Buttons */}
              <div className="mt-8 pt-6 border-t border-border flex items-center justify-between gap-4">
                <button
                  type="button"
                  onClick={handleBack}
                  disabled={currentStep === 0}
                  className="flex items-center gap-2 px-5 py-2.5 border border-border text-foreground rounded-md hover:bg-accent disabled:opacity-50 disabled:cursor-not-allowed transition-all font-medium"
                >
                  <ChevronLeft className="w-4 h-4" />
                  Back
                </button>

                {currentStep < STEPS.length - 1 ? (
                  <button
                    type="button"
                    onClick={handleNext}
                    className="flex items-center gap-2 px-6 py-2.5 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-all font-medium shadow-sm hover:shadow-md"
                  >
                    Next
                    <ChevronRight className="w-4 h-4" />
                  </button>
                ) : (
                  <button
                    type="button"
                    onClick={handleSubmit}
                    disabled={isSubmitting}
                    className="flex items-center gap-2 px-8 py-2.5 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all font-medium shadow-sm hover:shadow-md"
                  >
                    {isSubmitting ? (
                      <>
                        <Loader2 className="w-4 h-4 animate-spin" />
                        Creating...
                      </>
                    ) : (
                      <>
                        <Check className="w-4 h-4" />
                        Create Project
                      </>
                    )}
                  </button>
                )}
              </div>
            </div>
          </div>

          {/* Preview Card */}
          <div className="lg:col-span-1">
            <PreviewCard data={formData} />
          </div>
        </div>
      </div>
    </div>
  );
}
