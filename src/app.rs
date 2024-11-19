use crate::{
    auth::{LoginForm, LowboyLoginView, LowboyRegisterView, RegistrationForm},
    context::CloneableAppContext,
    controller,
    model::{LowboyUserRecord, LowboyUserTrait},
    view::LowboyLayout,
};
use axum::Router;
use serde::{Deserialize, Serialize};

#[allow(unused_variables)]
pub trait App<AC: CloneableAppContext>: Send + 'static {
    type User: LowboyUserTrait<LowboyUserRecord>;
    type Layout: LowboyLayout<Self::User>;
    type RegistrationForm: RegistrationForm + Clone + Serialize + for<'de> Deserialize<'de>;
    type RegisterView: LowboyRegisterView<Self::RegistrationForm>;
    type LoginForm: LoginForm + Clone + Serialize + for<'de> Deserialize<'de>;
    type LoginView: LowboyLoginView<Self::LoginForm>;

    fn name() -> &'static str;

    fn layout(context: &AC) -> Self::Layout {
        Self::Layout::default()
    }

    fn register_view(context: &AC) -> Self::RegisterView;

    fn login_view(context: &AC) -> Self::LoginView;

    fn routes() -> Router<AC>;

    fn auth_routes<App: self::App<AC>>() -> Router<AC> {
        controller::auth::routes::<App, AC>()
    }
}
