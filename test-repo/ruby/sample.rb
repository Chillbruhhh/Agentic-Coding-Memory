# Ruby sample file for AMP parser testing.
# Tests: classes, modules, methods, require/require_relative

require 'time'
require 'json'

# Module definitions
module AMP
  module TestRepo
    # Constants
    MAX_NAME_LENGTH = 100
    MAX_EMAIL_LENGTH = 200

    # Status module with constants
    module Status
      ACTIVE = 'active'
      INACTIVE = 'inactive'
      PENDING = 'pending'
      SUSPENDED = 'suspended'
    end

    # UserRole module
    module UserRole
      ADMIN = 'admin'
      USER = 'user'
      GUEST = 'guest'
    end

    # Validator module (mixin)
    module Validator
      def validate
        errors.empty?
      end

      def errors
        @errors ||= []
      end

      def add_error(message)
        errors << message
      end

      def clear_errors
        @errors = []
      end
    end

    # Logger module
    module Logger
      def log_info(message)
        puts "[INFO] #{Time.now.strftime('%Y-%m-%d %H:%M:%S')} - #{message}"
      end

      def log_error(message)
        warn "[ERROR] #{Time.now.strftime('%Y-%m-%d %H:%M:%S')} - #{message}"
      end

      def log_warning(message)
        puts "[WARN] #{Time.now.strftime('%Y-%m-%d %H:%M:%S')} - #{message}"
      end
    end

    # Base entity class
    class BaseEntity
      attr_accessor :id, :created_at, :updated_at

      def initialize
        @id = nil
        @created_at = Time.now
        @updated_at = Time.now
      end

      def valid?
        raise NotImplementedError, 'Subclasses must implement valid?'
      end

      def to_h
        {
          id: @id,
          created_at: @created_at.iso8601,
          updated_at: @updated_at.iso8601
        }
      end
    end

    # User class
    class User < BaseEntity
      include Validator

      attr_accessor :name, :email, :status, :role

      def initialize(name = nil, email = nil)
        super()
        @name = name
        @email = email
        @status = Status::ACTIVE
        @role = UserRole::USER
        @errors = []
      end

      def active?
        @status == Status::ACTIVE
      end

      def valid?
        clear_errors

        add_error('Name cannot be empty') if @name.nil? || @name.strip.empty?
        add_error('Invalid email format') if @email.nil? || !@email.include?('@')

        validate
      end

      def update_status(new_status)
        @status = new_status
        @updated_at = Time.now
      end

      def suspend(reason)
        @status = Status::SUSPENDED
        @updated_at = Time.now
        puts "User #{@name} suspended: #{reason}"
      end

      def promote_to_admin
        @role = UserRole::ADMIN
        @updated_at = Time.now
      end

      def to_h
        super.merge(
          name: @name,
          email: @email,
          status: @status,
          role: @role
        )
      end

      def to_json(*args)
        to_h.to_json(*args)
      end

      def to_s
        "User{id=#{@id}, name='#{@name}', email='#{@email}', status=#{@status}}"
      end
    end

    # Repository class
    class UserRepository
      include Logger

      def initialize
        @users = {}
        @next_id = 1
      end

      def find_by_id(id)
        @users[id]
      end

      def save(user)
        unless user.valid?
          raise ArgumentError, "Invalid user data: #{user.errors.join(', ')}"
        end

        if user.id.nil?
          user.id = @next_id
          @next_id += 1
          user.created_at = Time.now
        end

        user.updated_at = Time.now
        @users[user.id] = user

        log_info("User saved: #{user.name} (ID: #{user.id})")
        user
      end

      def delete(id)
        if @users.delete(id)
          log_info("User deleted: ID #{id}")
          true
        else
          false
        end
      end

      def find_all
        @users.values
      end

      def find_active_users
        @users.values.select(&:active?)
      end

      def find_by_role(role)
        @users.values.select { |user| user.role == role }
      end

      def count
        @users.size
      end

      def clear
        @users.clear
        @next_id = 1
      end
    end

    # Service class
    class UserService
      include Logger

      def initialize(repository)
        @repository = repository
      end

      def create_user(name, email)
        user = User.new(name, email)
        @repository.save(user)
      end

      def get_user_by_id(id)
        @repository.find_by_id(id)
      end

      def delete_user(id)
        @repository.delete(id)
      end

      def get_all_users
        @repository.find_all
      end

      def promote_to_admin(user_id)
        user = @repository.find_by_id(user_id)
        if user
          user.promote_to_admin
          @repository.save(user)
          log_info("User #{user.name} promoted to ADMIN")
        else
          log_error("User not found: ID #{user_id}")
        end
      end

      def suspend_user(user_id, reason)
        user = @repository.find_by_id(user_id)
        if user
          user.suspend(reason)
          @repository.save(user)
        else
          log_error("User not found: ID #{user_id}")
        end
      end

      def get_active_users
        @repository.find_active_users
      end
    end

    # Utility module
    module ValidationHelper
      module_function

      def valid_email?(email)
        !email.nil? && email.include?('@') && email.include?('.')
      end

      def valid_name?(name)
        !name.nil? && !name.strip.empty? && name.length >= 2
      end

      def sanitize_input(input)
        input.nil? ? '' : input.strip
      end
    end

    # Helper class with class methods
    class UserHelper
      class << self
        def format_user_display(user)
          "#{user.name} (#{user.email})"
        end

        def user_summary(users)
          {
            total: users.size,
            active: users.count(&:active?),
            inactive: users.count { |u| !u.active? }
          }
        end

        def export_to_json(users)
          users.map(&:to_h).to_json
        end
      end
    end

    # Custom error class
    class UserNotFoundError < StandardError
      attr_reader :user_id

      def initialize(user_id, message = 'User not found')
        @user_id = user_id
        super("#{message}: ID #{user_id}")
      end
    end
  end
end

# Main execution
if __FILE__ == $PROGRAM_NAME
  include AMP::TestRepo

  repository = UserRepository.new
  service = UserService.new(repository)

  # Create users
  user1 = service.create_user('Alice', 'alice@example.com')
  user2 = service.create_user('Bob', 'bob@example.com')

  puts "Created user: #{user1}"
  puts "Created user: #{user2}"

  # Promote user to admin
  service.promote_to_admin(user1.id)

  # List all users
  puts "\nAll users:"
  service.get_all_users.each do |user|
    puts "  #{user}"
  end

  # Get active users
  active_users = service.get_active_users
  puts "\nActive users: #{active_users.size}"

  # User summary
  summary = UserHelper.user_summary(service.get_all_users)
  puts "\nUser summary: #{summary}"
end
