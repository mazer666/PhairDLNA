//! The SOAP dispatcher.
//!
//! What: the routing of a control request to the handler of its action.
//! Why: the dispatcher picks the service from the namespace of the action
//!       element and the action from its name, and it answers an unknown
//!       service, an unknown action or a bad argument with the fault the
//!       specification names — `401 Invalid Action`, `402 Invalid Args` —
//!       instead of failing the request (`INT-025`, `INT-040`, `INT-048`).
//!       The dispatcher never panics (`UX-005`).
//! Example: `dispatch` answers a browse request with the browse answer, and a
//!       request for an action that does not exist with the fault `401`.

use crate::upnp::connection_manager::{
    connection_complete, get_current_connection_ids, get_current_connection_info,
    get_protocol_info, prepare_for_connection,
};
use crate::upnp::content_directory::{
    browse, get_search_capabilities, get_sort_capabilities, get_system_update_id, search,
};
use crate::upnp::content_model::ContentSource;
use crate::upnp::error::UpnpError;
use crate::upnp::services::{CONNECTION_MANAGER, CONTENT_DIRECTORY};
use crate::upnp::soap::{check_soap_action, parse_request, SoapRequest, SoapResponse};

/// The dispatcher of the control path.
pub struct Dispatcher<'a> {
    content: &'a dyn ContentSource,
}

impl<'a> Dispatcher<'a> {
    /// Builds a dispatcher that reads the catalogue through the given source.
    pub fn new(content: &'a dyn ContentSource) -> Self {
        Self { content }
    }

    /// Reads a control request and answers it.
    ///
    /// A request that cannot be read is answered with the fault `402 Invalid
    /// Args`, because a client that sent a broken envelope deserves to learn
    /// that, and not a dropped connection.
    pub fn dispatch(&self, body: &str, soap_action: Option<&str>) -> SoapResponse {
        match parse_request(body) {
            Ok(request) => {
                if let Err(error) = check_soap_action(&request, soap_action) {
                    return SoapResponse::from_error(&error);
                }
                self.dispatch_request(&request)
            }
            Err(error) => SoapResponse::from_error(&error),
        }
    }

    /// Answers a request that was already read.
    pub fn dispatch_request(&self, request: &SoapRequest) -> SoapResponse {
        match self.route(request) {
            Ok(response) => response,
            Err(error) => SoapResponse::from_error(&error),
        }
    }

    /// Finds the handler of the request.
    fn route(&self, request: &SoapRequest) -> Result<SoapResponse, UpnpError> {
        if request.service_type == CONTENT_DIRECTORY.service_type {
            return self.route_content_directory(request);
        }
        if request.service_type == CONNECTION_MANAGER.service_type {
            return self.route_connection_manager(request);
        }
        Err(UpnpError::UnknownService {
            service_type: request.service_type.clone(),
        })
    }

    /// Finds the handler of a request of the ContentDirectory service.
    fn route_content_directory(&self, request: &SoapRequest) -> Result<SoapResponse, UpnpError> {
        match request.action.as_str() {
            "GetSearchCapabilities" => Ok(get_search_capabilities()),
            "GetSortCapabilities" => Ok(get_sort_capabilities()),
            "GetSystemUpdateID" => Ok(get_system_update_id(self.content)),
            "Browse" => browse(request, self.content),
            "Search" => search(request, self.content),
            _ => Err(UpnpError::UnknownAction {
                action: request.action.clone(),
            }),
        }
    }

    /// Finds the handler of a request of the ConnectionManager service.
    fn route_connection_manager(&self, request: &SoapRequest) -> Result<SoapResponse, UpnpError> {
        match request.action.as_str() {
            "GetProtocolInfo" => Ok(get_protocol_info()),
            "GetCurrentConnectionIDs" => Ok(get_current_connection_ids()),
            "GetCurrentConnectionInfo" => get_current_connection_info(request),
            "PrepareForConnection" => prepare_for_connection(request),
            "ConnectionComplete" => connection_complete(request),
            _ => Err(UpnpError::UnknownAction {
                action: request.action.clone(),
            }),
        }
    }
}
