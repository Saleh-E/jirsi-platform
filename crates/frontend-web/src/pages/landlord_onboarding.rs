//! Landlord KYC Onboarding Component
//!
//! Provides a step-by-step onboarding flow for landlords to:
//! 1. Verify identity
//! 2. Connect Stripe account
//! 3. Add bank details
//! 4. Complete verification

use leptos::*;
use leptos_router::use_navigate;
use uuid::Uuid;

/// Onboarding step
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OnboardingStep {
    Welcome,
    BusinessInfo,
    StripeConnect,
    BankDetails,
    Verification,
    Complete,
}

impl OnboardingStep {
    fn index(&self) -> usize {
        match self {
            Self::Welcome => 0,
            Self::BusinessInfo => 1,
            Self::StripeConnect => 2,
            Self::BankDetails => 3,
            Self::Verification => 4,
            Self::Complete => 5,
        }
    }
    
    fn title(&self) -> &'static str {
        match self {
            Self::Welcome => "Welcome",
            Self::BusinessInfo => "Business Information",
            Self::StripeConnect => "Connect Payments",
            Self::BankDetails => "Bank Account",
            Self::Verification => "Verification",
            Self::Complete => "Complete",
        }
    }
    
    fn next(&self) -> Self {
        match self {
            Self::Welcome => Self::BusinessInfo,
            Self::BusinessInfo => Self::StripeConnect,
            Self::StripeConnect => Self::BankDetails,
            Self::BankDetails => Self::Verification,
            Self::Verification => Self::Complete,
            Self::Complete => Self::Complete,
        }
    }
    
    fn prev(&self) -> Self {
        match self {
            Self::Welcome => Self::Welcome,
            Self::BusinessInfo => Self::Welcome,
            Self::StripeConnect => Self::BusinessInfo,
            Self::BankDetails => Self::StripeConnect,
            Self::Verification => Self::BankDetails,
            Self::Complete => Self::Verification,
        }
    }
}

/// Landlord onboarding data
#[derive(Clone, Default)]
struct OnboardingData {
    // Business info
    business_type: String,
    business_name: String,
    legal_name: String,
    tax_id: String,
    
    // Contact
    phone: String,
    address: String,
    city: String,
    country: String,
    
    // Stripe
    stripe_account_id: Option<String>,
    stripe_onboarding_url: Option<String>,
    
    // Verification
    verified: bool,
}

/// Landlord KYC Onboarding Component
#[component]
pub fn LandlordOnboarding() -> impl IntoView {
    let (step, set_step) = create_signal(OnboardingStep::Welcome);
    let (data, set_data) = create_signal(OnboardingData::default());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);
    
    let navigate = use_navigate();
    
    let on_complete = move |_| {
        navigate("/app/settings/payments", Default::default());
    };
    
    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900">
            // Progress bar
            <div class="fixed top-0 left-0 right-0 z-50">
                <div class="h-1 bg-slate-700">
                    <div 
                        class="h-full bg-gradient-to-r from-indigo-500 to-purple-500 transition-all duration-300"
                        style=move || format!("width: {}%", (step.get().index() + 1) * 100 / 6)
                    />
                </div>
            </div>
            
            // Main content
            <div class="container mx-auto px-4 py-16 max-w-2xl">
                // Step indicator
                <div class="flex items-center justify-center gap-2 mb-8">
                    {(0..6).map(|i| view! {
                        <div 
                            class="w-3 h-3 rounded-full transition-all duration-300"
                            class:bg-indigo-500=move || step.get().index() >= i
                            class:bg-slate-600=move || step.get().index() < i
                            class:scale-125=move || step.get().index() == i
                        />
                    }).collect::<Vec<_>>()}
                </div>
                
                // Step title
                <h1 class="text-3xl font-bold text-white text-center mb-2">
                    {move || step.get().title()}
                </h1>
                <p class="text-slate-400 text-center mb-8">
                    {move || match step.get() {
                        OnboardingStep::Welcome => "Let's set up your landlord account to receive payments",
                        OnboardingStep::BusinessInfo => "Tell us about your business or personal details",
                        OnboardingStep::StripeConnect => "Connect your Stripe account to receive payments",
                        OnboardingStep::BankDetails => "Add your bank account for payouts",
                        OnboardingStep::Verification => "Verify your identity to complete setup",
                        OnboardingStep::Complete => "You're all set! Start receiving payments",
                    }}
                </p>
                
                // Error display
                {move || error.get().map(|e| view! {
                    <div class="bg-red-500/10 border border-red-500/30 text-red-400 px-4 py-3 rounded-lg mb-6">
                        {e}
                    </div>
                })}
                
                // Step content
                <div class="bg-white/5 backdrop-blur-lg border border-white/10 rounded-2xl p-8">
                    {move || match step.get() {
                        OnboardingStep::Welcome => view! {
                            <WelcomeStep />
                        }.into_view(),
                        OnboardingStep::BusinessInfo => view! {
                            <BusinessInfoStep data=data set_data=set_data />
                        }.into_view(),
                        OnboardingStep::StripeConnect => view! {
                            <StripeConnectStep data=data set_data=set_data loading=loading set_loading=set_loading set_error=set_error />
                        }.into_view(),
                        OnboardingStep::BankDetails => view! {
                            <BankDetailsStep data=data />
                        }.into_view(),
                        OnboardingStep::Verification => view! {
                            <VerificationStep data=data set_data=set_data />
                        }.into_view(),
                        OnboardingStep::Complete => view! {
                            <CompleteStep />
                        }.into_view(),
                    }}
                </div>
                
                // Navigation buttons
                <div class="flex justify-between mt-8">
                    <button
                        class="px-6 py-3 rounded-lg font-medium transition-all"
                        class:opacity-50=move || step.get() == OnboardingStep::Welcome
                        class:cursor-not-allowed=move || step.get() == OnboardingStep::Welcome
                        class:text-slate-400=move || step.get() != OnboardingStep::Welcome
                        class:hover:text-white=move || step.get() != OnboardingStep::Welcome
                        disabled=move || step.get() == OnboardingStep::Welcome
                        on:click=move |_| set_step.set(step.get().prev())
                    >
                        "← Back"
                    </button>
                    
                    {move || if step.get() == OnboardingStep::Complete {
                        view! {
                            <button
                                class="px-8 py-3 bg-gradient-to-r from-green-500 to-emerald-500 text-white rounded-lg font-medium hover:shadow-lg hover:shadow-green-500/25 transition-all"
                                on:click=on_complete
                            >
                                "Go to Dashboard →"
                            </button>
                        }.into_view()
                    } else {
                        view! {
                            <button
                                class="px-8 py-3 bg-gradient-to-r from-indigo-500 to-purple-500 text-white rounded-lg font-medium hover:shadow-lg hover:shadow-indigo-500/25 transition-all disabled:opacity-50"
                                disabled=loading
                                on:click=move |_| set_step.set(step.get().next())
                            >
                                {move || if loading.get() { "Loading..." } else { "Continue →" }}
                            </button>
                        }.into_view()
                    }}
                </div>
            </div>
        </div>
    }
}

/// Welcome step component
#[component]
fn WelcomeStep() -> impl IntoView {
    view! {
        <div class="text-center py-8">
            <div class="text-6xl mb-6">"🏠"</div>
            <h2 class="text-2xl font-bold text-white mb-4">"Become a Verified Landlord"</h2>
            <p class="text-slate-400 mb-6">
                "Complete this quick setup to start receiving rent payments directly to your bank account."
            </p>
            <div class="grid grid-cols-2 gap-4 text-left">
                <div class="bg-white/5 p-4 rounded-lg">
                    <div class="text-indigo-400 text-xl mb-2">"💳"</div>
                    <h3 class="font-medium text-white">"Secure Payments"</h3>
                    <p class="text-sm text-slate-400">"Powered by Stripe"</p>
                </div>
                <div class="bg-white/5 p-4 rounded-lg">
                    <div class="text-indigo-400 text-xl mb-2">"⚡"</div>
                    <h3 class="font-medium text-white">"Fast Payouts"</h3>
                    <p class="text-sm text-slate-400">"2-day transfers"</p>
                </div>
                <div class="bg-white/5 p-4 rounded-lg">
                    <div class="text-indigo-400 text-xl mb-2">"📊"</div>
                    <h3 class="font-medium text-white">"Track Everything"</h3>
                    <p class="text-sm text-slate-400">"Real-time dashboard"</p>
                </div>
                <div class="bg-white/5 p-4 rounded-lg">
                    <div class="text-indigo-400 text-xl mb-2">"🔒"</div>
                    <h3 class="font-medium text-white">"Bank-level Security"</h3>
                    <p class="text-sm text-slate-400">"256-bit encryption"</p>
                </div>
            </div>
        </div>
    }
}

/// Business info step
#[component]
fn BusinessInfoStep(
    data: ReadSignal<OnboardingData>,
    set_data: WriteSignal<OnboardingData>,
) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div>
                <label class="block text-sm font-medium text-slate-300 mb-2">"Business Type"</label>
                <select 
                    class="w-full bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-white focus:border-indigo-500 focus:outline-none"
                    on:change=move |e| {
                        let value = event_target_value(&e);
                        set_data.update(|d| d.business_type = value);
                    }
                >
                    <option value="individual">"Individual / Sole Proprietor"</option>
                    <option value="company">"Company / LLC"</option>
                    <option value="non_profit">"Non-Profit Organization"</option>
                </select>
            </div>
            
            <div>
                <label class="block text-sm font-medium text-slate-300 mb-2">"Legal Name"</label>
                <input 
                    type="text"
                    class="w-full bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-white placeholder-slate-500 focus:border-indigo-500 focus:outline-none"
                    placeholder="Your legal name or business name"
                    prop:value=move || data.get().legal_name.clone()
                    on:input=move |e| {
                        let value = event_target_value(&e);
                        set_data.update(|d| d.legal_name = value);
                    }
                />
            </div>
            
            <div>
                <label class="block text-sm font-medium text-slate-300 mb-2">"Phone Number"</label>
                <input 
                    type="tel"
                    class="w-full bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-white placeholder-slate-500 focus:border-indigo-500 focus:outline-none"
                    placeholder="+1 234 567 8900"
                    prop:value=move || data.get().phone.clone()
                    on:input=move |e| {
                        let value = event_target_value(&e);
                        set_data.update(|d| d.phone = value);
                    }
                />
            </div>
            
            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm font-medium text-slate-300 mb-2">"City"</label>
                    <input 
                        type="text"
                        class="w-full bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-white placeholder-slate-500 focus:border-indigo-500 focus:outline-none"
                        placeholder="City"
                        prop:value=move || data.get().city.clone()
                        on:input=move |e| {
                            let value = event_target_value(&e);
                            set_data.update(|d| d.city = value);
                        }
                    />
                </div>
                <div>
                    <label class="block text-sm font-medium text-slate-300 mb-2">"Country"</label>
                    <select 
                        class="w-full bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-white focus:border-indigo-500 focus:outline-none"
                        on:change=move |e| {
                            let value = event_target_value(&e);
                            set_data.update(|d| d.country = value);
                        }
                    >
                        <option value="US">"United States"</option>
                        <option value="AE">"UAE"</option>
                        <option value="GB">"United Kingdom"</option>
                        <option value="CA">"Canada"</option>
                        <option value="AU">"Australia"</option>
                    </select>
                </div>
            </div>
        </div>
    }
}

/// Stripe Connect step
#[component]
fn StripeConnectStep(
    data: ReadSignal<OnboardingData>,
    set_data: WriteSignal<OnboardingData>,
    loading: ReadSignal<bool>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let connect_stripe = move |_| {
        set_loading.set(true);
        set_error.set(None);
        
        // In production, this would call the backend to create a Stripe Connect account
        // and get an onboarding URL
        spawn_local(async move {
            // Simulate API call
            gloo_timers::future::TimeoutFuture::new(1500).await;
            
            // Mock success
            set_data.update(|d| {
                d.stripe_account_id = Some("acct_mock123".to_string());
                d.stripe_onboarding_url = Some("https://connect.stripe.com/setup/mock".to_string());
            });
            set_loading.set(false);
        });
    };
    
    view! {
        <div class="text-center py-8">
            {move || if data.get().stripe_account_id.is_some() {
                view! {
                    <div class="text-6xl mb-6">"✅"</div>
                    <h2 class="text-xl font-bold text-white mb-4">"Stripe Connected!"</h2>
                    <p class="text-slate-400">
                        "Your Stripe account is linked. Continue to add your bank details."
                    </p>
                }.into_view()
            } else {
                view! {
                    <div class="text-6xl mb-6">"💳"</div>
                    <h2 class="text-xl font-bold text-white mb-4">"Connect Your Stripe Account"</h2>
                    <p class="text-slate-400 mb-6">
                        "We use Stripe to securely process payments. Click below to connect or create an account."
                    </p>
                    <button
                        class="px-8 py-4 bg-[#635BFF] hover:bg-[#4D47D4] text-white rounded-lg font-medium transition-all inline-flex items-center gap-2 disabled:opacity-50"
                        disabled=loading
                        on:click=connect_stripe
                    >
                        {move || if loading.get() {
                            "Connecting..."
                        } else {
                            "Connect with Stripe →"
                        }}
                    </button>
                }.into_view()
            }}
        </div>
    }
}

/// Bank details step
#[component]
fn BankDetailsStep(data: ReadSignal<OnboardingData>) -> impl IntoView {
    view! {
        <div class="text-center py-8">
            <div class="text-6xl mb-6">"🏦"</div>
            <h2 class="text-xl font-bold text-white mb-4">"Bank Account Connected"</h2>
            <p class="text-slate-400 mb-6">
                "Your bank account details are securely stored with Stripe. Payouts will be sent automatically."
            </p>
            <div class="bg-white/5 rounded-lg p-4 inline-block">
                <p class="text-slate-400 text-sm">"Connected Bank"</p>
                <p class="text-white font-medium">"•••• •••• •••• 4242"</p>
            </div>
        </div>
    }
}

/// Verification step
#[component]
fn VerificationStep(
    data: ReadSignal<OnboardingData>,
    set_data: WriteSignal<OnboardingData>,
) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="text-center">
                <div class="text-6xl mb-4">"🔍"</div>
                <h2 class="text-xl font-bold text-white mb-2">"Verify Your Identity"</h2>
                <p class="text-slate-400">"This helps us comply with regulations and protect your account."</p>
            </div>
            
            <div class="space-y-4">
                <div class="bg-white/5 p-4 rounded-lg flex items-center gap-4">
                    <div class="w-12 h-12 bg-green-500/20 rounded-full flex items-center justify-center text-green-400">
                        "✓"
                    </div>
                    <div>
                        <p class="font-medium text-white">"Email Verified"</p>
                        <p class="text-sm text-slate-400">"Your email has been verified"</p>
                    </div>
                </div>
                
                <div class="bg-white/5 p-4 rounded-lg flex items-center gap-4">
                    <div class="w-12 h-12 bg-green-500/20 rounded-full flex items-center justify-center text-green-400">
                        "✓"
                    </div>
                    <div>
                        <p class="font-medium text-white">"Phone Verified"</p>
                        <p class="text-sm text-slate-400">"Your phone number has been verified"</p>
                    </div>
                </div>
                
                <div class="bg-white/5 p-4 rounded-lg flex items-center gap-4">
                    <div class="w-12 h-12 bg-yellow-500/20 rounded-full flex items-center justify-center text-yellow-400">
                        "⏳"
                    </div>
                    <div>
                        <p class="font-medium text-white">"ID Verification"</p>
                        <p class="text-sm text-slate-400">"Stripe will verify your identity (usually instant)"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Complete step
#[component]
fn CompleteStep() -> impl IntoView {
    view! {
        <div class="text-center py-8">
            <div class="text-8xl mb-6">"🎉"</div>
            <h2 class="text-2xl font-bold text-white mb-4">"You're All Set!"</h2>
            <p class="text-slate-400 mb-6">
                "Your landlord account is now fully set up. You can start receiving payments immediately."
            </p>
            <div class="grid grid-cols-3 gap-4 text-center">
                <div class="bg-white/5 p-4 rounded-lg">
                    <p class="text-3xl font-bold text-white">"0"</p>
                    <p class="text-sm text-slate-400">"Properties"</p>
                </div>
                <div class="bg-white/5 p-4 rounded-lg">
                    <p class="text-3xl font-bold text-white">"$0"</p>
                    <p class="text-sm text-slate-400">"Collected"</p>
                </div>
                <div class="bg-white/5 p-4 rounded-lg">
                    <p class="text-3xl font-bold text-green-400">"Active"</p>
                    <p class="text-sm text-slate-400">"Status"</p>
                </div>
            </div>
        </div>
    }
}
