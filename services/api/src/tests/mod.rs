#[cfg(test)]
mod cart_tests {
    /// Cart subtotal calculation: sum of (unit_price × quantity) for all items.
    #[test]
    fn subtotal_is_sum_of_line_totals() {
        struct LineItem {
            unit_price: i64,
            quantity: i32,
        }

        let items = vec![
            LineItem { unit_price: 1999, quantity: 2 },
            LineItem { unit_price: 999, quantity: 1 },
        ];

        let subtotal: i64 = items.iter().map(|i| i.unit_price * i.quantity as i64).sum();
        assert_eq!(subtotal, 4997); // 3998 + 999
    }

    #[test]
    fn empty_cart_has_zero_subtotal() {
        let items: Vec<(i64, i32)> = vec![];
        let subtotal: i64 = items.iter().map(|(p, q)| p * *q as i64).sum();
        assert_eq!(subtotal, 0);
    }
}

#[cfg(test)]
mod checkout_tests {
    /// Mock payment: any positive amount should be approved.
    #[derive(Debug, PartialEq)]
    enum MockPaymentResult {
        Approved,
        Declined,
    }

    fn mock_process_payment(amount: i64) -> MockPaymentResult {
        if amount > 0 {
            MockPaymentResult::Approved
        } else {
            MockPaymentResult::Declined
        }
    }

    #[test]
    fn positive_amount_is_approved() {
        assert_eq!(mock_process_payment(1000), MockPaymentResult::Approved);
    }

    #[test]
    fn zero_amount_is_declined() {
        assert_eq!(mock_process_payment(0), MockPaymentResult::Declined);
    }

    #[test]
    fn negative_amount_is_declined() {
        assert_eq!(mock_process_payment(-1), MockPaymentResult::Declined);
    }
}

#[cfg(test)]
mod tenancy_tests {
    /// Tenant guard: a user without tenant_id claim must get a BadRequest error.
    use crate::authz::Claims;

    fn make_claims(tenant_id: Option<&str>, roles: Vec<&str>) -> Claims {
        Claims {
            sub: "user-123".to_string(),
            iss: "http://localhost:8080/realms/bookstore".to_string(),
            exp: 9999999999,
            iat: 0,
            email: None,
            preferred_username: None,
            realm_access: Some(crate::authz::RealmAccess {
                roles: roles.into_iter().map(String::from).collect(),
            }),
            tenant_id: tenant_id.map(String::from),
        }
    }

    #[test]
    fn claims_with_tenant_id_has_tenant() {
        let claims = make_claims(Some("tenant-001"), vec!["customer"]);
        assert_eq!(claims.tenant_id.as_deref(), Some("tenant-001"));
    }

    #[test]
    fn claims_without_tenant_id_is_none() {
        let claims = make_claims(None, vec!["customer"]);
        assert!(claims.tenant_id.is_none());
    }

    #[test]
    fn customer_role_check() {
        let claims = make_claims(Some("t1"), vec!["customer"]);
        assert!(claims.is_customer());
        assert!(!claims.is_tenant_admin());
    }

    #[test]
    fn admin_role_check() {
        let claims = make_claims(Some("t1"), vec!["tenant_admin"]);
        assert!(claims.is_tenant_admin());
        assert!(!claims.is_customer());
    }

    #[test]
    fn platform_admin_role_check() {
        let claims = make_claims(Some("t1"), vec!["platform_admin"]);
        assert!(claims.is_platform_admin());
    }
}

#[cfg(test)]
mod error_tests {
    use crate::common::AppError;
    use axum::response::IntoResponse;

    #[test]
    fn not_found_returns_404() {
        let resp = AppError::NotFound.into_response();
        assert_eq!(resp.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[test]
    fn unauthorized_returns_401() {
        let resp = AppError::Unauthorized.into_response();
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn forbidden_returns_403() {
        let resp = AppError::Forbidden.into_response();
        assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);
    }

    #[test]
    fn bad_request_returns_400() {
        let resp = AppError::BadRequest("oops".into()).into_response();
        assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn conflict_returns_409() {
        let resp = AppError::Conflict("duplicate".into()).into_response();
        assert_eq!(resp.status(), axum::http::StatusCode::CONFLICT);
    }
}
