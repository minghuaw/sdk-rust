use fe2o3_amqp_types::primitives::{SimpleValue, Symbol, Binary};
use fe2o3_amqp_types::messaging::{Data as AmqpData};

use crate::message::StructuredSerializer;
use crate::{message::{BinarySerializer, MessageAttributeValue, Error}, event::SpecVersion};

use super::constants::DATACONTENTTYPE;
use super::{AmqpCloudEvent, ATTRIBUTE_PREFIX, AmqpBody};

impl BinarySerializer<AmqpCloudEvent> for AmqpCloudEvent {
    fn set_spec_version(mut self, spec_version: SpecVersion) -> crate::message::Result<Self> {
        let key = String::from("cloudEvents:specversion");
        let value = String::from(spec_version.as_str());
        self.application_properties.insert(key, SimpleValue::from(value));
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> crate::message::Result<Self> {
        // For the binary mode, the AMQP content-type property field value maps directly to the
        // CloudEvents datacontenttype attribute.
        // 
        // All CloudEvents attributes with exception of datacontenttype MUST be individually mapped
        // to and from the AMQP application-properties section.
        if name == DATACONTENTTYPE {
            self.content_type = match value {
                MessageAttributeValue::String(s) => Some(Symbol::from(s)),
                _ => return Err(Error::WrongEncoding {  })
            }
        } else {
            // CloudEvent attributes are prefixed with "cloudEvents:" for use in the
            // application-properties section
            let key = format!("{}:{}", ATTRIBUTE_PREFIX, name);
            let value = SimpleValue::from(value);
            self.application_properties.insert(key, value);
        }

        Ok(self)
    }

    // Extension attributes are always serialized according to binding rules like standard
    // attributes. However this specification does not prevent an extension from copying event
    // attribute values to other parts of a message, in order to interact with non-CloudEvents
    // systems that also process the message. Extension specifications that do this SHOULD specify
    // how receivers are to interpret messages if the copied values differ from the cloud-event
    // serialized values.
    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> crate::message::Result<Self> {
        let key = format!("{}:{}", ATTRIBUTE_PREFIX, name);
        let value = SimpleValue::from(value);
        self.application_properties.insert(key, value);
        Ok(self)
    }

    fn end_with_data(mut self, bytes: Vec<u8>) -> crate::message::Result<Self> {
        let data = Binary::from(bytes);
        self.body = AmqpBody::Data(AmqpData(data));
        Ok(self)
    }

    fn end(self) -> crate::message::Result<Self> {
        Ok(self)
    }
}

impl StructuredSerializer<AmqpCloudEvent> for AmqpCloudEvent {
    fn set_structured_event(mut self, bytes: Vec<u8>) -> crate::message::Result<Self> {
        self.content_type = Some(Symbol::from("application/cloudevents+json; charset=utf-8"));
        self.body = AmqpBody::Data(AmqpData(Binary::from(bytes)));
        Ok(self)
    }
}